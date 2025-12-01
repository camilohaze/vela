# TASK-000F: Especificación Formal del Lenguaje Vela

## 📋 Información General
- **Historia:** VELA-561 (Formal Specifications - Phase 0)
- **Epic:** EPIC-00B: Formal Specifications
- **Sprint:** 1
- **Estado:** Pendiente ⏳
- **Prioridad:** P0 (Crítica)
- **Estimación:** 80 horas

---

## 🎯 Objetivo

Crear una especificación formal completa del lenguaje Vela con el mismo rigor que Rust Reference o ECMAScript Specification, documentando:

- **Lexical structure** (tokens, keywords, operators)
- **Type system formal rules** (tipos primitivos, compuestos, generics)
- **Operational semantics** (cómo se ejecutan las construcciones)
- **Expression evaluation order** (order of evaluation)
- **Statement execution semantics** (control flow, side effects)
- **Function call semantics** (parámetros, returns, closures)

---

## 📐 Especificación Formal

### 1. Lexical Structure

#### 1.1 Character Set
```ebnf
Source-File ::= UTF-8-BOM? Source-Text
Source-Text ::= Line*
Line ::= Character* (Newline | EOF)
Character ::= Unicode-Scalar-Value
Newline ::= \n | \r\n | \r
```

#### 1.2 Tokens
```ebnf
Token ::= Keyword | Identifier | Literal | Operator | Delimiter | Comment

Keyword ::= "fn" | "class" | "struct" | "enum" | "type" | "interface"
          | "state" | "computed" | "memo" | "effect" | "watch"
          | "import" | "show" | "hide" | "as" | "public" | "private" | "protected"
          | "if" | "else" | "match" | "return" | "throw" | "try" | "catch" | "finally"
          | "async" | "await" | "yield" | "constructor" | "this" | "super"
          | "extends" | "implements" | "override" | "overload" | "abstract"
          | "mount" | "update" | "destroy" | "beforeUpdate" | "afterUpdate"
          | "true" | "false" | "None"
          | "Number" | "Float" | "String" | "Bool" | "void" | "never"
          | "Option" | "Result" | "Some" | "Ok" | "Err"
          
          /* Keywords específicos por dominio */
          | "widget" | "component" | "service" | "repository" | "controller"
          | "usecase" | "entity" | "dto" | "valueObject" | "model"
          | "factory" | "builder" | "strategy" | "observer" | "singleton"
          | "adapter" | "decorator" | "guard" | "middleware" | "interceptor"
          | "validator" | "module" | "store" | "provider" | "actor"
          | "pipe" | "task" | "helper" | "mapper" | "serializer"

Identifier ::= IdentifierStart IdentifierContinue*
IdentifierStart ::= [a-z A-Z _]
IdentifierContinue ::= [a-z A-Z 0-9 _]

Literal ::= NumberLiteral | StringLiteral | BooleanLiteral

NumberLiteral ::= DecimalLiteral | HexLiteral | BinaryLiteral | OctalLiteral
DecimalLiteral ::= [0-9]+ ("." [0-9]+)? (("e"|"E") ("+"|"-")? [0-9]+)?
HexLiteral ::= "0x" [0-9a-fA-F]+
BinaryLiteral ::= "0b" [01]+
OctalLiteral ::= "0o" [0-7]+

StringLiteral ::= '"' StringChar* '"'
StringChar ::= EscapeSequence | StringInterpolation | [^"\n\r\\]
EscapeSequence ::= "\\" ("n" | "r" | "t" | "\\" | '"' | "0")
StringInterpolation ::= "${" Expression "}"

BooleanLiteral ::= "true" | "false"

Operator ::= "+" | "-" | "*" | "/" | "%" | "**"
           | "==" | "!=" | "<" | ">" | "<=" | ">="
           | "&&" | "||" | "!"
           | "=" | "+=" | "-=" | "*=" | "/=" | "%="
           | "&" | "|" | "^" | "<<" | ">>"
           | "." | "?." | ".." | "..="
           | "=>" | "->" | "::" | "@"

Delimiter ::= "(" | ")" | "{" | "}" | "[" | "]"
            | "," | ";" | ":"

Comment ::= LineComment | BlockComment
LineComment ::= "//" [^\n\r]* (Newline | EOF)
BlockComment ::= "/*" (Character | Newline)* "*/"
```

---

### 2. Type System Formal Rules

#### 2.1 Type Grammar
```ebnf
Type ::= PrimitiveType
       | CompositeType
       | FunctionType
       | GenericType
       | UnionType
       | OptionType
       | ResultType

PrimitiveType ::= "Number" | "Float" | "String" | "Bool" | "void" | "never"

CompositeType ::= StructType | EnumType | ClassType | InterfaceType
StructType ::= "struct" Identifier TypeParams? "{" FieldList "}"
EnumType ::= "enum" Identifier TypeParams? "{" VariantList "}"
ClassType ::= "class" Identifier TypeParams? ("extends" Type)? ("implements" Type)? "{" MemberList "}"
InterfaceType ::= "interface" Identifier TypeParams? "{" MemberList "}"

FunctionType ::= "fn" "(" ParamTypeList? ")" "->" Type
ParamTypeList ::= ParamType ("," ParamType)*
ParamType ::= Identifier ":" Type

GenericType ::= Identifier "<" TypeArgList ">"
TypeArgList ::= Type ("," Type)*
TypeParams ::= "<" TypeParam ("," TypeParam)* ">"
TypeParam ::= Identifier (":" TypeBound)?
TypeBound ::= Type ("+" Type)*

UnionType ::= Type "|" Type ("|" Type)*

OptionType ::= "Option" "<" Type ">"
ResultType ::= "Result" "<" Type "," Type ">"
```

#### 2.2 Type Checking Rules

**Regla 1: Inmutabilidad por defecto**
```
Γ ⊢ x: T
-----------------  (no state keyword)
x es inmutable
```

**Regla 2: Mutabilidad explícita**
```
Γ ⊢ state x: T
-----------------
x es mutable y reactivo
```

**Regla 3: Inferencia de tipos (Hindley-Milner)**
```
Γ ⊢ e: T₁    T₁ ~ T₂
-----------------------  (Unification)
Γ ⊢ e: T₂
```

**Regla 4: Option<T> safety**
```
Γ ⊢ e: Option<T>
Γ, x: T ⊢ body: U
---------------------------------  (if-let)
Γ ⊢ if let Some(x) = e { body }: Option<U>
```

**Regla 5: Result<T, E> propagation**
```
Γ ⊢ e: Result<T, E>
Γ, x: T ⊢ body: Result<U, E>
---------------------------------  (try)
Γ ⊢ e.andThen(x => body): Result<U, E>
```

**Regla 6: Function subtyping**
```
T₁' <: T₁    T₂ <: T₂'
---------------------------------  (Contravariant args, Covariant return)
(T₁ -> T₂) <: (T₁' -> T₂')
```

**Regla 7: Generic instantiation**
```
Γ ⊢ e: ∀α. T    Γ ⊢ U type
---------------------------------
Γ ⊢ e: T[α := U]
```

---

### 3. Operational Semantics

#### 3.1 Expression Evaluation

**Arithmetic Expressions**
```
⟨n₁, σ⟩ ⇓ v₁    ⟨n₂, σ⟩ ⇓ v₂
---------------------------------  (E-Add)
⟨n₁ + n₂, σ⟩ ⇓ v₁ + v₂
```

**Function Call**
```
⟨fn, σ⟩ ⇓ closure(params, body, env)
⟨args, σ⟩ ⇓ vals
env' = env[params := vals]
⟨body, env'⟩ ⇓ v
---------------------------------  (E-Call)
⟨fn(args), σ⟩ ⇓ v
```

**Pattern Matching (exhaustivo)**
```
⟨e, σ⟩ ⇓ v
∃ pattern_i tal que v matches pattern_i
⟨branch_i, σ[bindings]⟩ ⇓ v'
---------------------------------  (E-Match)
⟨match e { ... pattern_i => branch_i ... }, σ⟩ ⇓ v'
```

**Option<T> unwrap**
```
⟨e, σ⟩ ⇓ Some(v)
---------------------------------  (E-Unwrap-Some)
⟨e.unwrap(), σ⟩ ⇓ v

⟨e, σ⟩ ⇓ None
---------------------------------  (E-Unwrap-None)
⟨e.unwrap(), σ⟩ ⇓ panic("unwrap on None")
```

#### 3.2 Statement Execution

**Variable Declaration (inmutable)**
```
⟨expr, σ⟩ ⇓ v
σ' = σ[x := (v, immutable)]
---------------------------------  (S-Let)
⟨x: T = expr, σ⟩ → σ'
```

**State Declaration (mutable + reactivo)**
```
⟨expr, σ⟩ ⇓ v
σ' = σ[x := (v, mutable, reactive)]
signal(x) registrado en sistema reactivo
---------------------------------  (S-State)
⟨state x: T = expr, σ⟩ → σ'
```

**Assignment (solo mutable)**
```
σ(x) = (_, mutable, _)
⟨expr, σ⟩ ⇓ v
σ' = σ[x := v]
notify_watchers(x)
---------------------------------  (S-Assign)
⟨x = expr, σ⟩ → σ'
```

**If Statement**
```
⟨cond, σ⟩ ⇓ true    ⟨then_branch, σ⟩ → σ'
---------------------------------  (S-If-True)
⟨if cond { then_branch }, σ⟩ → σ'

⟨cond, σ⟩ ⇓ false    ⟨else_branch, σ⟩ → σ'
---------------------------------  (S-If-False)
⟨if cond { ... } else { else_branch }, σ⟩ → σ'
```

**Return Statement**
```
⟨expr, σ⟩ ⇓ v
---------------------------------  (S-Return)
⟨return expr, σ⟩ ⇓ Return(v)
```

---

### 4. Expression Evaluation Order

Vela garantiza **left-to-right evaluation** en todas las expresiones:

```vela
# Evaluación de izquierda a derecha GARANTIZADA
result = f() + g() + h()
# Orden: f() primero, luego g(), luego h()

# Assignments también left-to-right
x = y = z = 0
# Equivalente a: z = 0; y = z; x = y
```

**Regla formal:**
```
Para expr₁ op expr₂:
1. Evaluar expr₁ → v₁
2. Evaluar expr₂ → v₂  
3. Aplicar op(v₁, v₂)
```

**Short-circuit evaluation:**
```
# AND lógico
⟨e₁, σ⟩ ⇓ false
---------------------------------  (E-And-Short)
⟨e₁ && e₂, σ⟩ ⇓ false

# OR lógico
⟨e₁, σ⟩ ⇓ true
---------------------------------  (E-Or-Short)
⟨e₁ || e₂, σ⟩ ⇓ true
```

---

### 5. Statement Execution Semantics

#### 5.1 Control Flow

**If-Else como expresión:**
```
⟨cond, σ⟩ ⇓ true    ⟨then_expr, σ⟩ ⇓ v
---------------------------------  (E-If-Expr-True)
⟨if cond { then_expr } else { else_expr }, σ⟩ ⇓ v

⟨cond, σ⟩ ⇓ false    ⟨else_expr, σ⟩ ⇓ v
---------------------------------  (E-If-Expr-False)
⟨if cond { then_expr } else { else_expr }, σ⟩ ⇓ v
```

**Match exhaustivo (obligatorio):**
```
patterns = {p₁, p₂, ..., pₙ}
∀ value: T, ∃ pᵢ tal que value matches pᵢ
---------------------------------  (Exhaustiveness)
match expr: T { p₁ => ..., p₂ => ..., pₙ => ... }  OK
```

#### 5.2 Side Effects

**Effect declarativo:**
```
state x: Number = 0

effect {
  print("x changed to ${x}")
}

# Ejecuta automáticamente cuando x cambia
x = 5  # Trigger: "x changed to 5"
```

**Regla formal:**
```
σ(x) = (v_old, mutable, reactive)
watchers(x) = {effect₁, effect₂, ...}
σ' = σ[x := v_new]
∀ effectᵢ ∈ watchers(x): execute(effectᵢ)
---------------------------------  (S-State-Update)
⟨x = v_new, σ⟩ → σ'
```

---

### 6. Function Call Semantics

#### 6.1 Parameter Passing

**Vela usa pass-by-value para tipos copiables, pass-by-reference para otros:**

```vela
# Tipos primitivos: pass-by-value (copy)
fn increment(x: Number) -> Number {
  return x + 1
}
n = 5
result = increment(n)  # n sigue siendo 5

# Tipos complejos: pass-by-reference (borrow)
fn modifyList(list: List<Number>) -> void {
  list.push(10)  # Modifica lista original
}
myList = [1, 2, 3]
modifyList(myList)  # myList ahora es [1, 2, 3, 10]
```

**Regla formal:**
```
T ∈ {Number, Float, String, Bool}
---------------------------------  (Pass-By-Value)
parámetro copiado

T ∉ {Number, Float, String, Bool}
---------------------------------  (Pass-By-Reference)
parámetro prestado (borrow)
```

#### 6.2 Closures

```vela
fn makeCounter() -> fn() -> Number {
  state count: Number = 0
  
  return () => {
    count = count + 1
    return count
  }
}

counter = makeCounter()
counter()  # 1
counter()  # 2
```

**Regla formal:**
```
env = {x₁: v₁, x₂: v₂, ...}  # Entorno capturado
free_vars(body) = {x₁, x₂, ...}
---------------------------------  (E-Closure)
⟨fn() { body }, env⟩ ⇓ closure(body, env[free_vars])
```

#### 6.3 Async/Await

```vela
async fn fetchData() -> Result<String, Error> {
  response = await httpClient.get("https://api.example.com")
  return Ok(response.body)
}
```

**Regla formal:**
```
⟨async_fn(), σ⟩ ⇓ Future<T>

⟨await future, σ⟩ suspend hasta que future se resuelva
future resuelve a Ok(v)
---------------------------------  (E-Await)
⟨await future, σ⟩ ⇓ v
```

---

### 7. Memory Model

#### 7.1 Ownership y Borrowing

```
Regla 1: Cada valor tiene exactamente un owner
Regla 2: Cuando el owner sale de scope, el valor se drop
Regla 3: Referencias inmutables: múltiples lectores
Regla 4: Referencias mutables: un único escritor
```

**Ejemplo:**
```vela
# Ownership transfer
list1 = [1, 2, 3]
list2 = list1  # list1 ya no es válido (moved)

# Borrowing (referencia inmutable)
fn sum(list: &List<Number>) -> Number {
  # list es prestado, no puede modificar
}

# Borrowing mutable
fn appendZero(list: &mut List<Number>) -> void {
  list.push(0)
}
```

#### 7.2 Automatic Reference Counting (ARC)

```
Cada objeto tiene ref_count
ref_count++ cuando se crea nueva referencia
ref_count-- cuando referencia sale de scope
if ref_count == 0 → deallocate
```

---

### 8. Concurrency Model

#### 8.1 Actor System

```vela
actor UserService {
  state users: List<User> = []
  
  fn addUser(user: User) -> void {
    this.users.push(user)
  }
  
  fn getUsers() -> List<User> {
    return this.users
  }
}

# Mensajes son encolados y procesados secuencialmente
service = UserService()
service.addUser(user1)  # Mensaje 1
service.addUser(user2)  # Mensaje 2
```

**Garantías:**
- Un actor procesa un mensaje a la vez (no race conditions)
- Orden de mensajes preservado
- Aislamiento de estado (no shared mutable state)

#### 8.2 Reactive Signals

```vela
state count: Number = 0

computed doubled: Number {
  return count * 2
}

effect {
  print("Count: ${count}, Doubled: ${doubled}")
}

count = 5  # Trigger: compute doubled → execute effect
```

**Garantías:**
- Updates propagados en orden topológico
- No circular dependencies (error de compilación)
- Batch updates (una vez por tick)

---

## 📊 Comparación con Otros Lenguajes

| Característica | Vela | Rust | TypeScript | Dart |
|----------------|------|------|------------|------|
| **Type System** | Hindley-Milner | Ownership | Structural | Nominal |
| **Mutabilidad** | Inmutable por defecto | Inmutable por defecto | Mutable | Mutable |
| **Null Safety** | Option<T> | Option<T> | null/undefined | null | 
| **Async Model** | async/await | async/await | async/await | async/await |
| **Reactive** | Built-in (signals) | No | No (libraries) | No (streams) |
| **Memory** | ARC | Ownership | GC | GC |
| **Concurrency** | Actors | Threads | Workers | Isolates |

---

## ✅ Criterios de Aceptación

- [x] Especificación léxica completa (EBNF)
- [x] Type system con reglas formales
- [x] Operational semantics definida
- [x] Evaluation order especificado
- [x] Statement semantics documentada
- [x] Function call semantics completa
- [x] Memory model formal
- [x] Concurrency model documentado

---

## 🔗 Referencias

### Especificaciones de Referencia
- [Rust Reference](https://doc.rust-lang.org/reference/)
- [ECMAScript Specification](https://tc39.es/ecma262/)
- [The Go Programming Language Specification](https://go.dev/ref/spec)
- [Dart Language Specification](https://dart.dev/guides/language/spec)

### Papers Académicos
- **Type Inference:** Hindley-Milner Type System
- **Memory Safety:** Region-Based Memory Management
- **Concurrency:** Actor Model (Agha, 1986)
- **Reactive Programming:** Functional Reactive Programming (Elliott, 1997)

---

**Estado:** ⏳ Pendiente de implementación  
**Prioridad:** P0 - Bloqueante para desarrollo serio del compilador  
**Siguiente paso:** TASK-000G (Modelo de memoria formal detallado)
