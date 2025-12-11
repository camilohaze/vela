# 1. Diseño de Gramática y Sintaxis Concreta de Vela

## 1.1 Gramática Formal (EBNF)

### 1.1.1 Estructura de Programa

```ebnf
Program          ::= ModuleDecl? ImportDecl* TopLevelDecl*
ModuleDecl       ::= 'module' QualifiedName ';'
ImportDecl       ::= 'import' ImportSpec (',' ImportSpec)* ';'
ImportSpec       ::= QualifiedName ('as' Identifier)? ('{' ImportList '}')?
ImportList       ::= Identifier (',' Identifier)*
TopLevelDecl     ::= FunctionDecl | ClassDecl | InterfaceDecl | EnumDecl | TypeAliasDecl 
                   | ConstDecl | ExtensionDecl | ActorDecl
```

### 1.1.2 Declaraciones de Tipos

```ebnf
TypeAliasDecl    ::= Visibility? 'type' Identifier GenericParams? '=' Type ';'
EnumDecl         ::= Visibility? 'enum' Identifier GenericParams? '{' EnumVariant* '}'
EnumVariant      ::= Identifier ('(' Type (',' Type)* ')')? ','?
InterfaceDecl    ::= Visibility? 'interface' Identifier GenericParams? ExtendsList? '{' InterfaceMember* '}'
InterfaceMember  ::= MethodSignature | PropertySignature
MethodSignature  ::= Identifier GenericParams? '(' ParamList? ')' (':' Type)? ';'
PropertySignature::= Identifier ':' Type ';'
```

### 1.1.3 Declaraciones de Clases

```ebnf
ClassDecl        ::= Visibility? Modifier* 'class' Identifier GenericParams? 
                     ExtendsList? ImplementsList? '{' ClassMember* '}'
Modifier         ::= 'abstract' | 'final' | 'sealed'
ExtendsList      ::= 'extends' Type
ImplementsList   ::= 'implements' Type (',' Type)*
ClassMember      ::= Constructor | Method | Property | OperatorOverload
Constructor      ::= Identifier '(' ParamList? ')' Block
Method           ::= Visibility? MethodModifier* Identifier GenericParams? 
                     '(' ParamList? ')' (':' Type)? (Block | '=>' Expr ';')
MethodModifier   ::= 'override' | 'async' | 'static' | 'abstract'
Property         ::= Visibility? PropertyModifier* Identifier ':' Type ('=' Expr)? ';'
PropertyModifier ::= 'static' | 'readonly' | 'late'
OperatorOverload ::= 'operator' Operator '(' ParamList ')' ':' Type (Block | '=>' Expr ';')
```

### 1.1.4 Extensiones (estilo Swift)

```ebnf
ExtensionDecl    ::= 'extension' Type ForClause? '{' ExtensionMember* '}'
ForClause        ::= 'for' Type ('where' Constraint (',' Constraint)*)?
ExtensionMember  ::= Method | Property | OperatorOverload
```

### 1.1.5 Funciones

```ebnf
FunctionDecl     ::= Visibility? 'fn' Identifier GenericParams? 
                     '(' ParamList? ')' (':' Type)? (Block | '=>' Expr ';')
ParamList        ::= Param (',' Param)*
Param            ::= Identifier ':' Type ('=' Expr)?
```

### 1.1.6 Actors (Concurrencia)

```ebnf
ActorDecl        ::= Visibility? 'actor' Identifier GenericParams? '{' ActorMember* '}'
ActorMember      ::= Method | Property | MessageHandler
MessageHandler   ::= 'on' Identifier '(' ParamList? ')' Block
```

### 1.1.7 Sistema de Tipos

```ebnf
Type             ::= PrimaryType ('|' PrimaryType)*                    // Union types
PrimaryType      ::= BaseType                                         // NO nullable (usar Option<T>)
BaseType         ::= Identifier GenericArgs?
                   | FunctionType
                   | TupleType
                   | ArrayType
                   | SignalType
                   | ActorType
                   | OptionType
GenericArgs      ::= '<' Type (',' Type)* '>'
FunctionType     ::= '(' TypeList? ')' '=>' Type
TupleType        ::= '(' Type (',' Type)+ ')'
ArrayType        ::= Type '[' ']'
SignalType       ::= 'Signal' '<' Type '>'
ActorType        ::= 'Actor' '<' Type '>'
OptionType       ::= 'Option' '<' Type '>'                            // Null-safety
GenericParams    ::= '<' GenericParam (',' GenericParam)* '>'
GenericParam     ::= Identifier Constraint*
Constraint       ::= ':' Type | 'where' Expr
```

**Nota**: No existe `null` ni tipos nullable `?` - usar `Option<T>` para valores opcionales.

### 1.1.8 Expresiones

```ebnf
Expr             ::= AssignmentExpr
AssignmentExpr   ::= LogicalOrExpr (('=' | '+=' | '-=' | '*=' | '/=') LogicalOrExpr)?
LogicalOrExpr    ::= LogicalAndExpr ('||' LogicalAndExpr)*
LogicalAndExpr   ::= EqualityExpr ('&&' EqualityExpr)*
EqualityExpr     ::= RelationalExpr (('==' | '!=') RelationalExpr)*
RelationalExpr   ::= AdditiveExpr (('<' | '>' | '<=' | '>=') AdditiveExpr)*
AdditiveExpr     ::= MultiplicativeExpr (('+' | '-') MultiplicativeExpr)*
MultiplicativeExpr ::= UnaryExpr (('*' | '/' | '%') UnaryExpr)*
UnaryExpr        ::= ('!' | '-' | '+' | 'await')? PostfixExpr
PostfixExpr      ::= PrimaryExpr (CallSuffix | IndexSuffix | MemberSuffix)*
CallSuffix       ::= '(' ArgList? ')'
IndexSuffix      ::= '[' Expr ']'
MemberSuffix     ::= '.' Identifier
PrimaryExpr      ::= Literal | Identifier | ParenExpr | IfExpr | MatchExpr 
                   | LambdaExpr | ListExpr | DictExpr | SignalExpr | UIExpr
```

### 1.1.9 Expresiones Reactivas (Signals)

```ebnf
SignalExpr       ::= SignalDecl | ComputedDecl | EffectDecl | WatchDecl
SignalDecl       ::= 'signal' '(' Expr ')'
ComputedDecl     ::= 'computed' '(' Lambda ')'
EffectDecl       ::= 'effect' '(' Lambda ')'
WatchDecl        ::= 'watch' '(' Expr ',' Lambda ')'
```

### 1.1.10 UI Declarativa (Flutter-style)

```ebnf
UIExpr           ::= StatefulWidget | StatelessWidget
StatefulWidget   ::= Identifier WidgetArgs? '{' StatefulBody '}'
StatelessWidget  ::= Identifier WidgetArgs? '{' WidgetBody '}'
WidgetArgs       ::= '(' ArgList ')'
WidgetBody       ::= WidgetProp* UIExpr*
StatefulBody     ::= StateDecl* WidgetProp* UIExpr*
StateDecl        ::= 'state' Identifier ':' Type '=' Expr ';'
WidgetProp       ::= Identifier ':' Expr ','?
```

Ejemplo sintaxis (StatelessWidget):
```vela
Container {
  padding: 16,
  color: Colors.blue,
  
  Column {
    Text("Hello"),
    Button {
      label: "Click me",
      onClick: () => print("Clicked")
    }
  }
}
```

Ejemplo sintaxis (StatefulWidget):
```vela
fn Counter(): StatefulWidget {
  state count: Int = 0;
  
  return Column {
    Text("Count: ${count}"),
    Button {
      label: "Increment",
      onClick: () => state { count = count + 1; }
    }
  };
}
```

### 1.1.11 Statements

```ebnf
Block            ::= '{' Statement* '}'
Statement        ::= VarDecl | ExprStmt | ReturnStmt | IfStmt
                   | MatchStmt | TryStmt | StateBlock | EffectStmt
VarDecl          ::= Identifier (':' Type)? '=' Expr ';'          // Inmutable por defecto
StateMutation    ::= 'state' Identifier ':' Type '=' Expr ';'     // Mutable con state
ExprStmt         ::= Expr ';'
ReturnStmt       ::= 'return' Expr? ';'
IfStmt           ::= 'if' '(' Expr ')' Block ('else' (IfStmt | Block))?
MatchStmt        ::= 'match' '(' Expr ')' '{' MatchArm* '}'
MatchArm         ::= Pattern (MatchGuard)? '=>' (Expr | Block) ','?
MatchGuard       ::= 'if' Expr
TryStmt          ::= 'try' Block CatchClause* FinallyClause?
CatchClause      ::= 'catch' '(' Identifier ':' Type ')' Block
FinallyClause    ::= 'finally' Block
StateBlock       ::= 'state' Block
EffectStmt       ::= 'effect' Block
```

**Nota**: No existen `for`, `while`, `loop`, `const` ni `let` - Vela es puramente funcional.

### 1.1.12 Patterns (Pattern Matching)

```ebnf
Pattern          ::= LiteralPattern | IdentifierPattern | TuplePattern 
                   | VariantPattern | ListPattern | WildcardPattern
LiteralPattern   ::= Literal
IdentifierPattern::= Identifier
TuplePattern     ::= '(' Pattern (',' Pattern)* ')'
VariantPattern   ::= QualifiedName ('(' Pattern (',' Pattern)* ')')?
ListPattern      ::= '[' Pattern (',' Pattern)* ']'
WildcardPattern  ::= '_'
```

### 1.1.13 Literales

```ebnf
Literal          ::= IntLiteral | FloatLiteral | StringLiteral | BoolLiteral | CharLiteral
IntLiteral       ::= [0-9]+ | '0x'[0-9a-fA-F]+ | '0b'[01]+
FloatLiteral     ::= [0-9]+ '.' [0-9]+ ([eE][+-]?[0-9]+)?
StringLiteral    ::= '"' StringChar* '"' | "'" StringChar* "'"
BoolLiteral      ::= 'true' | 'false'
CharLiteral      ::= "'" . "'"
```

**Nota**: No existe `null` - usar `Option.None` para valores ausentes.

### 1.1.14 Identificadores

```ebnf
Identifier       ::= [a-zA-Z_][a-zA-Z0-9_]*
QualifiedName    ::= Identifier ('.' Identifier)*
Visibility       ::= 'public' | 'private' | 'protected' | 'internal'
```

---

## 1.2 Precedencia de Operadores

| Precedencia | Operadores | Asociatividad |
|-------------|-----------|---------------|
| 1 (más alta) | `()`, `[]`, `.` | Izquierda |
| 2 | `!`, `-`, `+` (unario), `await` | Derecha |
| 3 | `*`, `/`, `%` | Izquierda |
| 4 | `+`, `-` | Izquierda |
| 5 | `<`, `>`, `<=`, `>=` | Izquierda |
| 6 | `==`, `!=` | Izquierda |
| 7 | `&&` | Izquierda |
| 8 | `||` | Izquierda |
| 9 | `=`, `+=`, `-=`, `*=`, `/=` | Derecha |

---

## 1.3 Palabras Reservadas

```
abstract    actor       as          async       await       
case        catch       class       computed    continue    
default     do          effect      else        enum           
extends     extension   false       finally     fn          
if          implements  import      in          interface   internal    
late        match       memo        module      mutable     
operator    override    package     private     protected   
public      readonly    return      sealed      signal      state       
static      super       switch      this        throw       true        
try         type        watch       where       worker      
yield
```

**Removidas**: `for`, `while`, `loop`, `break` (no loops), `const`, `let` (inmutable por defecto), `export` (usar modifiers), `null` (usar Option<T>)

---

## 1.4 Sistema de Imports con Prefijos

Vela usa **prefijos explícitos** para diferenciar el origen de las dependencias:

### Sintaxis de Imports

```ebnf
ImportDecl       ::= 'import' ImportPath ';'
ImportPath       ::= PrefixedPath | SimplePath
PrefixedPath     ::= 'system:' Path
                   | 'package:' Path
                   | 'module:' Path
                   | 'library:' Path
                   | 'extension:' Path
                   | 'assets:' Path
SimplePath       ::= Identifier ('/' Identifier)*
```

### Prefijos de Import

| Prefijo | Uso | Ejemplo |
|---------|-----|---------|
| `system:` | APIs internas de Vela | `import 'system:ui'` |
| `package:` | Dependencias externas instaladas | `import 'package:lodash'` |
| `module:` | Módulos del proyecto (@module) | `import 'module:auth'` |
| `library:` | Librerías internas (@library) | `import 'library:utils'` |
| `assets:` | Recursos estáticos | `import 'assets:logo.png'` |

### Ejemplos de Imports

```vela
# APIs internas de Vela (compilador provee)
import 'system:ui'              # Container, Widget, Column, Text, Button
import 'system:http'            # HttpClient, Request, Response
import 'system:reactive'        # signal, computed, effect
import 'system:actors'          # Actor, ActorSystem, Message
import 'system:state'           # Store, Action, Reducer

# Dependencias externas instaladas (vela.yaml)
import 'package:lodash'         # Librería externa
import 'package:axios'          # Cliente HTTP externo
import 'package:date-fns'       # Utilidades de fecha
import 'package:jwt'            # JWT library

# Módulos del proyecto (definidos con @module)
import 'module:auth'            # AuthModule
import 'module:users'           # UsersModule
import 'module:auth/services'   # AuthService, AuthRepository
import 'module:auth/widgets'    # LoginWidget, RegisterWidget

# Librerías internas (definidas con @library)
import 'library:utils'          # Utilidades internas
import 'library:validators'     # Validadores internos
import 'library:helpers'        # Helpers internos

# Assets estáticos
import 'assets:images/logo.png'
import 'assets:fonts/roboto.ttf'
```

### Estructura de Directorios

```
proyecto/
├── src/                    → módulos (import 'module:X')
│   ├── auth/
│   │   ├── auth.module.vela
│   │   ├── services/
│   │   └── widgets/
│   └── users/
├── lib/                    → librerías (import 'library:X')
│   ├── utils.vela
│   └── validators.vela
├── extensions/             → extensiones (import 'extension:X')
│   ├── charts/
│   └── maps/
├── assets/                 → assets (import 'assets:X')
│   ├── images/
│   └── fonts/
└── vela.yaml               → dependencias externas (package:)
```

### Reglas

1. **`system:`** - APIs del compilador, NO requiere instalación
2. **`package:`** - DEBE estar en `vela.yaml` y ejecutar `vela add <package>`
3. **`module:`** - DEBE tener archivo `.module.vela` con decorador `@module`
4. **`library:`** - DEBE tener decorador `@library`
5. **`assets:`** - Archivos estáticos en carpeta `assets/`

---

## 1.5 Ejemplos de Sintaxis Completa

### 1.5.1 Archivo con Imports Completo

```vela
# APIs internas
import 'system:ui'
import 'system:http'
import 'system:reactive'

# Dependencias externas
import 'package:lodash'
import 'package:date-fns'

# Módulos del proyecto
import 'module:auth/services'

# Librerías internas
import 'library:validators'

@injectable
service UserService {
  constructor(@inject private authService: AuthService) { }
  
  public fn getUsers(): List<User> {
    return this.authService.getAllUsers()
  }
}
```

### 1.5.2 Función Pura

```vela
fn add(a: Int, b: Int): Int => a + b;

fn factorial(n: Int): Int {
  if (n <= 1) return 1;
  return n * factorial(n - 1);
}
```

### 1.5.3 Función con State

```vela
import 'system:reactive'

fn counter(): Int {
  count = signal(0);  # Inmutable por defecto
  
  state {
    count.value = count.value + 1;
  }
  
  return count.value;
}
```

### 1.4.4 Clase con OOP

```vela
public class Person {
  private name: String;
  private age: Int;
  
  Person(name: String, age: Int) {
    this.name = name;
    this.age = age;
  }
  
  public fn greet(): String => "Hello, I'm ${this.name}";
  
  public fn celebrateBirthday(): void {
    state {
      this.age += 1;
    }
  }
}

public class Employee extends Person {
  private salary: Float;
  
  Employee(name: String, age: Int, salary: Float) {
    super(name, age);
    this.salary = salary;
  }
  
  override fn greet(): String => "${super.greet()} and I'm an employee";
}
```

### 1.4.5 Interface y Polimorfismo

```vela
public interface Drawable {
  fn draw(): void;
  fn area(): Float;
}

public class Circle implements Drawable {
  private radius: Float;
  
  Circle(radius: Float) {
    this.radius = radius;
  }
  
  fn draw(): void {
    print("Drawing circle");
  }
  
  fn area(): Float => 3.14159 * this.radius * this.radius;
}
```

### 1.4.6 Generics

```vela
// Map está implementado como método funcional de List
// Ejemplo de uso:
fn processNumbers(list: List<Int>): List<Int> {
  return list.map(n => n * 2);
}

class Box<T> {
  private value: T;
  
  Box(value: T) {
    this.value = value;
  }
  
  fn get(): T => this.value;
  fn set(value: T): void {
    state {
      this.value = value;
    }
  }
}
```

### 1.4.7 Enums y ADTs

```vela
enum Result<T, E> {
  Ok(T),
  Err(E)
}

enum Option<T> {
  Some(T),
  None  // NO existe 'null', usar Option.None
}

fn divide(a: Float, b: Float): Result<Float, String> {
  if (b == 0.0) {
    return Result.Err("Division by zero");
  }
  return Result.Ok(a / b);
}

fn handleResult(result: Result<Float, String>): void {
  match (result) {
    Result.Ok(value) => print("Result: ${value}"),
    Result.Err(error) => print("Error: ${error}")
  }
}
```

### 1.4.8 Signals Reactivos

```vela
fn reactiveCounter(): StatelessWidget {
  count = signal(0);  // Inmutable por defecto
  doubled = computed(() => count.value * 2);
  
  effect(() => {
    print("Count changed: ${count.value}");
  });
  
  watch(count, (newValue, oldValue) => {
    print("Count changed from ${oldValue} to ${newValue}");
  });
  
  return Container {
    Column {
      Text("Count: ${count.value}"),
      Text("Doubled: ${doubled.value}"),
      Button {
        label: "Increment",
        onClick: () => state { count.value = count.value + 1; }
      }
    }
  };
}
```

### 1.4.9 Actor (Concurrencia)

```vela
actor Counter {
  private count: Int = 0;
  
  on increment() {
    this.count += 1;
  }
  
  on getCount(): Int {
    return this.count;
  }
  
  fn reset(): void {
    this.count = 0;
  }
}

async fn useCounter(): void {
  let counter = Counter();
  
  await counter.send(increment());
  await counter.send(increment());
  
  let value = await counter.send(getCount());
  print("Counter value: ${value}");
}
```

### 1.4.10 UI Declarativa Completa (StatefulWidget)

```vela
fn TodoApp(): StatefulWidget {
  todos = signal<List<String>>([]);
  input = signal("");
  
  fn addTodo(): void {
    state {
      if (input.value.trim() != "") {
        todos.value = todos.value.concat([input.value]);
        input.value = "";
      }
    }
  }
  
  return Container {
    padding: 20,
    
    Column {
      spacing: 16,
      
      Text("Todo App", style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold)),
      
      Row {
        spacing: 8,
        
        TextField {
          value: input,
          placeholder: "Enter a todo",
          onEnter: addTodo
        },
        
        Button {
          label: "Add",
          onClick: addTodo
        }
      },
      
      // Usar map en lugar de for loop
      ...todos.value.map(todo => TodoItem(text: todo))
    }
  };
}

fn TodoItem(text: String): StatefulWidget {
  completed = signal(false);
  
  return Row {
    spacing: 8,
    
    Checkbox {
      value: completed,
      onChange: (value) => state { completed.value = value; }
    },
    
    Text(
      text,
      style: TextStyle(
        decoration: completed.value ? TextDecoration.lineThrough : TextDecoration.none
      )
    )
  };
}
```

### 1.4.11 Async/Await

```vela
async fn fetchUser(id: Int): Result<User, Error> {
  try {
    response = await http.get("https://api.example.com/users/${id}");
    user = await response.json<User>();
    return Result.Ok(user);
  } catch (e: NetworkError) {
    return Result.Err(e);
  }
}

async fn loadUsers(): void {
  userIds = [1, 2, 3, 4, 5];
  users = await Promise.all(userIds.map((id) => fetchUser(id)));
  
  // Usar forEach en lugar de for loop
  users.forEach(result => {
    match (result) {
      Result.Ok(user) => print("User: ${user.name}"),
      Result.Err(error) => print("Failed to load user: ${error}")
    }
  });
}
```

### 1.4.12 Extensions

```vela
extension List<T> {
  fn first(): Option<T> {
    if (this.length > 0) {
      return Option.Some(this[0]);
    }
    return Option.None;
  }
  
  fn last(): Option<T> {
    if (this.length > 0) {
      return Option.Some(this[this.length - 1]);
    }
    return Option.None;
  }
}

extension Int {
  fn times(fn: (Int) => void): void {
    // Implementación funcional sin loops
    (0..this).forEach(fn);
  }
}

// Uso
5.times((i) => print(i)); // Imprime 0, 1, 2, 3, 4
```

### 1.4.13 Sobrecarga de Operadores

```vela
class Vector2 {
  public x: Float;
  public y: Float;
  
  Vector2(x: Float, y: Float) {
    this.x = x;
    this.y = y;
  }
  
  operator +(other: Vector2): Vector2 {
    return Vector2(this.x + other.x, this.y + other.y);
  }
  
  operator *(scalar: Float): Vector2 {
    return Vector2(this.x * scalar, this.y * scalar);
  }
  
  operator ==(other: Vector2): Bool {
    return this.x == other.x && this.y == other.y;
  }
}

// Uso - Inmutable por defecto (no necesita let)
v1 = Vector2(1.0, 2.0);
v2 = Vector2(3.0, 4.0);
v3 = v1 + v2;  // Vector2(4.0, 6.0)
v4 = v1 * 2.0; // Vector2(2.0, 4.0)
```

---

## 1.5 Comentarios

```vela
// Comentario de una línea

/*
  Comentario
  de múltiples
  líneas
*/

/// Comentario de documentación
/// Soporta Markdown
/// 
/// # Ejemplo
/// ```vela
/// let x = 42;
/// ```
fn documentedFunction(): void {
  // ...
}
```

---

## 1.6 String Interpolation

```vela
name = "Vela";  // Inmutable por defecto
version = 1.0;
message = "Welcome to ${name} version ${version}";

// Multi-line strings
html = """
  <div>
    <h1>${title}</h1>
    <p>${content}</p>
  </div>
""";
```

---

## 1.7 Lexer - Tokens Principales

```
KEYWORDS:     if, else, fn, class, return, let, const, etc.
IDENTIFIERS:  [a-zA-Z_][a-zA-Z0-9_]*
OPERATORS:    +, -, *, /, %, ==, !=, <, >, <=, >=, &&, ||, !, =, +=, -=, etc.
DELIMITERS:   (, ), {, }, [, ], ;, :, ,, .
LITERALS:     123, 3.14, "string", 'char', true, false, null
COMMENTS:     //, /* */
WHITESPACE:   space, tab, newline (ignorado excepto para separación)
```

---

## 1.8 Reglas de Visibilidad

- **`public`**: Accesible desde cualquier módulo
- **`private`**: Solo dentro de la clase/módulo actual
- **`protected`**: Clase actual y subclases
- **`internal`**: Dentro del mismo package

**Default**: Si no se especifica, es `internal`.

---

## 1.9 Null Safety con Option<T>

```vela
x: String = "hello";                    // Non-nullable siempre
y: Option<String> = Option.Some("hi"); // Valor opcional
z: Option<String> = Option.None;       // Sin valor (NO null)

// Safe access con pattern matching
length = match (y) {
  Option.Some(value) => Option.Some(value.length),
  Option.None => Option.None
};

// Unwrap con valor por defecto
name = user.name.unwrapOr("Anonymous");

// Pattern matching con Option
match (value) {
  Option.None => print("No value"),
  Option.Some(v) => print("Value: ${v}")
}
```

**Nota**: NO existe `null` ni tipos nullable `?`. Usar `Option<T>` para valores opcionales.

---

## 1.10 Type Inference

```vela
x = 42;              // Inferido como Int - Inmutable por defecto
y = 3.14;            // Inferido como Float
z = "hello";         // Inferido como String
list = [1, 2, 3];    // Inferido como List<Int>
dict = {"a": 1};     // Inferido como Dict<String, Int>

result = numbers.map((n) => n * 2);  // Inferido tipo de lambda y retorno
```

**Nota**: No necesita `let` ni `const` - todas las variables son inmutables por defecto.

---

## 1.11 Refinement Types (Ligeros)

```vela
type PositiveInt = Int where value > 0;
type NonEmptyString = String where value.length > 0;
type Email = String where value.matches(/^[^\s@]+@[^\s@]+\.[^\s@]+$/);

fn divide(a: Int, b: PositiveInt): Float {
  return a / b;  // Safe, b nunca es 0
}
```

---

## 1.12 Sintaxis de Módulos y Packages

```vela
// Archivo: src/main.vela
module com.example.myapp;

// Archivo: src/models/user.vela
module com.example.myapp.models;

// Archivo: src/ui/components/button.vela
module com.example.myapp.ui.components;
```

**Estructura de directorios**:
```
myapp/
├── vela.yaml (manifest)
├── src/
│   ├── main.vela
│   ├── models/
│   │   └── user.vela
│   └── ui/
│       └── components/
│           └── button.vela
└── test/
    └── main_test.vela
```

---

**FIN DEL DOCUMENTO: Gramática y Sintaxis Concreta de Vela**

Esta especificación cubre todos los aspectos sintácticos del lenguaje y servirá como base para:
- Implementación del Lexer
- Implementación del Parser
- Generación del AST
- Validación sintáctica
- Herramientas de formateo y linting
