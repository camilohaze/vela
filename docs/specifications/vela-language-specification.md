# Vela Language Specification (Formal)

**Version:** 0.1.0-draft  
**Status:** Work in Progress  
**Last Updated:** 2025-11-30

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Lexical Structure](#2-lexical-structure)
3. [Type System](#3-type-system)
4. [Operational Semantics](#4-operational-semantics)
5. [Expression Evaluation](#5-expression-evaluation)
6. [Statement Execution](#6-statement-execution)
7. [Function Call Semantics](#7-function-call-semantics)

---

## 1. Introduction

This document provides the formal specification of the Vela programming language. It follows the rigor of the [Rust Reference](https://doc.rust-lang.org/reference/) and [ECMAScript specification](https://tc39.es/ecma262/).

### 1.1 Notation

- **Grammar notation**: Extended Backus-Naur Form (EBNF)
- **Semantic rules**: Inference rules with premises and conclusions
- **Type judgments**: `Γ ⊢ e : τ` (under environment Γ, expression e has type τ)

---

## 2. Lexical Structure

### 2.1 Input Format

```
Input := UTF-8 encoded text
```

### 2.2 Lexical Grammar

#### 2.2.1 Whitespace

```ebnf
Whitespace := 
    | ' ' | '\t' | '\n' | '\r'

Comment := 
    | LineComment 
    | BlockComment

LineComment := '//' ~('\n')* '\n'
BlockComment := '/*' ~('*/')* '*/'
```

#### 2.2.2 Keywords

```ebnf
Keyword := 
    | 'fn' | 'let' | 'const' | 'var'
    | 'if' | 'else' | 'match' | 'loop' | 'while' | 'for' | 'break' | 'continue' | 'return'
    | 'type' | 'interface' | 'enum' | 'struct'
    | 'actor' | 'signal' | 'async' | 'await'
    | 'import' | 'export' | 'from'
    | 'true' | 'false' | 'null' | 'undefined'
    | 'this' | 'super'
```

**Semantic constraint:** Keywords are reserved and cannot be used as identifiers.

#### 2.2.3 Identifiers

```ebnf
Identifier := IdentifierStart IdentifierContinue*

IdentifierStart := 
    | Unicode_Letter 
    | '_'

IdentifierContinue := 
    | IdentifierStart 
    | Unicode_Digit
```

**Semantic constraint:** Identifiers must not collide with keywords.

#### 2.2.4 Literals

```ebnf
Literal := 
    | IntegerLiteral
    | FloatLiteral
    | StringLiteral
    | BooleanLiteral
    | NullLiteral

IntegerLiteral := 
    | DecimalInteger
    | HexInteger
    | OctalInteger
    | BinaryInteger

DecimalInteger := Digit+
HexInteger := '0x' HexDigit+
OctalInteger := '0o' OctalDigit+
BinaryInteger := '0b' BinaryDigit+

FloatLiteral := 
    | Digit+ '.' Digit+ Exponent?
    | Digit+ Exponent

Exponent := ('e' | 'E') ('+' | '-')? Digit+

StringLiteral := 
    | '"' StringChar* '"'
    | "'" StringChar* "'"

BooleanLiteral := 'true' | 'false'
NullLiteral := 'null'
```

#### 2.2.5 Operators and Punctuation

```ebnf
Operator := 
    | '+' | '-' | '*' | '/' | '%'
    | '==' | '!=' | '<' | '<=' | '>' | '>='
    | '&&' | '||' | '!'
    | '=' | '+=' | '-=' | '*=' | '/=' | '%='
    | '->' | '=>' | '<-'
    | '?' | ':'

Punctuation := 
    | '(' | ')' | '{' | '}' | '[' | ']'
    | ',' | ';' | '.' | '::'
```

---

## 3. Type System

### 3.1 Type Grammar

```ebnf
Type := 
    | PrimitiveType
    | FunctionType
    | TupleType
    | ArrayType
    | ObjectType
    | GenericType
    | UnionType
    | TypeVariable

PrimitiveType := 
    | 'Int' | 'Float' | 'String' | 'Bool' | 'Null' | 'Undefined'

FunctionType := '(' TypeList? ')' '->' Type

TupleType := '(' TypeList ')'

ArrayType := '[' Type ']'

ObjectType := '{' (Identifier ':' Type (',' Identifier ':' Type)*)? '}'

GenericType := Identifier '<' TypeList '>'

UnionType := Type '|' Type

TypeList := Type (',' Type)*
```

### 3.2 Type Inference Rules

#### 3.2.1 Variables

```
Γ(x) = τ
─────────────── (T-Var)
Γ ⊢ x : τ
```

#### 3.2.2 Literals

```
─────────────── (T-Int)
Γ ⊢ n : Int


─────────────── (T-Float)
Γ ⊢ f : Float


─────────────── (T-String)
Γ ⊢ s : String


─────────────── (T-Bool)
Γ ⊢ b : Bool


─────────────── (T-Null)
Γ ⊢ null : Null
```

#### 3.2.3 Functions

```
Γ, x₁:τ₁, ..., xₙ:τₙ ⊢ e : τ
────────────────────────────────────────── (T-Fn)
Γ ⊢ fn(x₁:τ₁, ..., xₙ:τₙ) -> τ { e } : (τ₁, ..., τₙ) -> τ
```

#### 3.2.4 Application

```
Γ ⊢ e₁ : (τ₁, ..., τₙ) -> τ    Γ ⊢ e₂ : τ₁    ...    Γ ⊢ eₙ : τₙ
───────────────────────────────────────────────────────────────── (T-App)
Γ ⊢ e₁(e₂, ..., eₙ) : τ
```

#### 3.2.5 Let Binding

```
Γ ⊢ e₁ : τ₁    Γ, x:τ₁ ⊢ e₂ : τ₂
────────────────────────────────── (T-Let)
Γ ⊢ let x = e₁; e₂ : τ₂
```

### 3.3 Subtyping Rules

```
────────── (S-Refl)
τ <: τ


τ₁ <: τ₂    τ₂ <: τ₃
──────────────────── (S-Trans)
τ₁ <: τ₃


τ₁ <: τ₁'    τ₂' <: τ₂
──────────────────────── (S-Fun)
(τ₁' -> τ₂') <: (τ₁ -> τ₂)
```

---

## 4. Operational Semantics

### 4.1 Evaluation Contexts

```
E := 
    | □                       (hole)
    | E op e                  (left operand)
    | v op E                  (right operand)
    | E(e₁, ..., eₙ)         (function)
    | v(v₁, ..., vᵢ₋₁, E, eᵢ₊₁, ..., eₙ)  (arguments)
    | let x = E; e           (let binding)
```

### 4.2 Small-Step Semantics

#### 4.2.1 Variable Lookup

```
σ(x) = v
──────────────── (E-Var)
⟨x, σ⟩ → ⟨v, σ⟩
```

#### 4.2.2 Let Binding

```
⟨e₁, σ⟩ → ⟨v, σ'⟩
──────────────────────────────── (E-Let)
⟨let x = e₁; e₂, σ⟩ → ⟨e₂, σ'[x ↦ v]⟩
```

#### 4.2.3 Function Application

```
⟨e₁, σ⟩ → ⟨λx.e, σ'⟩    ⟨e₂, σ'⟩ → ⟨v, σ''⟩
───────────────────────────────────────────── (E-App)
⟨e₁(e₂), σ⟩ → ⟨e[x ↦ v], σ''⟩
```

#### 4.2.4 Arithmetic Operations

```
⟨e₁, σ⟩ → ⟨n₁, σ'⟩    ⟨e₂, σ'⟩ → ⟨n₂, σ''⟩
──────────────────────────────────────────── (E-Add)
⟨e₁ + e₂, σ⟩ → ⟨n₁ + n₂, σ''⟩
```

---

## 5. Expression Evaluation

### 5.1 Evaluation Order

**Rule:** Left-to-right, call-by-value evaluation.

#### 5.1.1 Binary Operators

For binary expression `e₁ op e₂`:

1. Evaluate `e₁` to value `v₁`
2. Evaluate `e₂` to value `v₂`
3. Apply operator: `v₁ op v₂`

#### 5.1.2 Function Calls

For function call `f(e₁, ..., eₙ)`:

1. Evaluate `f` to closure `λ(x₁, ..., xₙ).e`
2. Evaluate arguments left-to-right: `e₁` → `v₁`, ..., `eₙ` → `vₙ`
3. Substitute: `e[x₁ ↦ v₁, ..., xₙ ↦ vₙ]`
4. Evaluate body

### 5.2 Short-Circuit Evaluation

#### 5.2.1 Logical AND

```
e₁ && e₂ ≡ if e₁ then e₂ else false
```

#### 5.2.2 Logical OR

```
e₁ || e₂ ≡ if e₁ then true else e₂
```

### 5.3 Side Effects

**Rule:** Side effects occur in evaluation order.

**Example:**
```vela
let x = (print("1"), 10);
let y = (print("2"), 20);
x + y;  // Prints "1", then "2"
```

---

## 6. Statement Execution

### 6.1 Control Flow

#### 6.1.1 If Statement

```
⟨cond, σ⟩ → ⟨true, σ'⟩    ⟨then_branch, σ'⟩ → ⟨v, σ''⟩
────────────────────────────────────────────────────── (S-If-True)
⟨if cond { then_branch } else { else_branch }, σ⟩ → ⟨v, σ''⟩


⟨cond, σ⟩ → ⟨false, σ'⟩    ⟨else_branch, σ'⟩ → ⟨v, σ''⟩
────────────────────────────────────────────────────── (S-If-False)
⟨if cond { then_branch } else { else_branch }, σ⟩ → ⟨v, σ''⟩
```

#### 6.1.2 While Loop

```
⟨cond, σ⟩ → ⟨false, σ'⟩
──────────────────────────── (S-While-End)
⟨while cond { body }, σ⟩ → ⟨(), σ'⟩


⟨cond, σ⟩ → ⟨true, σ'⟩    ⟨body, σ'⟩ → ⟨v, σ''⟩    ⟨while cond { body }, σ''⟩ → ⟨v', σ'''⟩
──────────────────────────────────────────────────────────────────────────────────────── (S-While-Continue)
⟨while cond { body }, σ⟩ → ⟨v', σ'''⟩
```

#### 6.1.3 Return Statement

```
⟨e, σ⟩ → ⟨v, σ'⟩
────────────────────── (S-Return)
⟨return e, σ⟩ → return(v, σ')
```

### 6.2 Exception Propagation

**Rule:** Exceptions propagate up the call stack until caught or program terminates.

```
⟨e, σ⟩ → error(msg, σ')
───────────────────────────── (S-Error-Prop)
⟨E[e], σ⟩ → error(msg, σ')
```

---

## 7. Function Call Semantics

### 7.1 Closure Creation

```
Γ ⊢ fn(x₁:τ₁, ..., xₙ:τₙ) -> τ { e }
───────────────────────────────────── (F-Closure)
⟨fn(x₁, ..., xₙ) { e }, σ⟩ → ⟨⟨λ(x₁, ..., xₙ).e, σ⟩, σ⟩
```

**Note:** Closures capture the environment at creation time.

### 7.2 Call Semantics

#### 7.2.1 Direct Call

```
⟨f, σ⟩ → ⟨⟨λ(x₁, ..., xₙ).e, σ_capture⟩, σ'⟩
⟨a₁, σ'⟩ → ⟨v₁, σ''⟩
...
⟨aₙ, σⁿ⁻¹⟩ → ⟨vₙ, σⁿ⟩
⟨e[x₁ ↦ v₁, ..., xₙ ↦ vₙ], σ_capture ∪ σⁿ⟩ → ⟨v, σ_result⟩
────────────────────────────────────────────────────────────── (F-Call)
⟨f(a₁, ..., aₙ), σ⟩ → ⟨v, σ_result⟩
```

#### 7.2.2 Tail Call Optimization

**Guarantee:** Tail calls do not grow the call stack.

A call is in tail position if:
1. It's the last expression in a function body
2. Its result is returned without modification

```vela
fn factorial(n: Int, acc: Int) -> Int {
    if n == 0 {
        return acc;  // Not a tail call (return wraps it)
    } else {
        factorial(n - 1, n * acc)  // Tail call (last expression)
    }
}
```

### 7.3 Recursion

**Guarantee:** The runtime supports unlimited recursion depth (subject to available stack space).

#### 7.3.1 Direct Recursion

```vela
fn fib(n: Int) -> Int {
    if n <= 1 {
        return n;
    } else {
        return fib(n - 1) + fib(n - 2);
    }
}
```

#### 7.3.2 Mutual Recursion

```vela
fn is_even(n: Int) -> Bool {
    if n == 0 { true } else { is_odd(n - 1) }
}

fn is_odd(n: Int) -> Bool {
    if n == 0 { false } else { is_even(n - 1) }
}
```

---

## 8. Type Soundness

### 8.1 Progress Theorem

**Theorem (Progress):** If `⊢ e : τ`, then either:
1. `e` is a value, or
2. There exists `e'` such that `e → e'`

### 8.2 Preservation Theorem

**Theorem (Preservation):** If `Γ ⊢ e : τ` and `e → e'`, then `Γ ⊢ e' : τ`.

### 8.3 Type Safety

**Corollary:** Well-typed programs do not get stuck (excluding runtime errors like division by zero).

---

## Appendix A: Reserved Keywords (Complete List)

```
fn          let         const       var         if          else
match       loop        while       for         break       continue
return      type        interface   enum        struct      actor
signal      async       await       import      export      from
true        false       null        undefined   this        super
try         catch       finally     throw       as          is
in          of          new         delete      typeof      void
yield       static      public      private     protected   abstract
final       override    virtual     extends     implements  package
namespace   module      class       trait       where       mut
ref         move        copy        unsafe      extern      macro
```

---

## Appendix B: Operator Precedence

| Level | Operators | Associativity |
|-------|-----------|---------------|
| 1 | `()` `[]` `.` | Left |
| 2 | `!` `-` (unary) | Right |
| 3 | `*` `/` `%` | Left |
| 4 | `+` `-` | Left |
| 5 | `<` `<=` `>` `>=` | Left |
| 6 | `==` `!=` | Left |
| 7 | `&&` | Left |
| 8 | `||` | Left |
| 9 | `=` `+=` `-=` `*=` `/=` | Right |

---

**References:**
- Rust Reference: https://doc.rust-lang.org/reference/
- ECMAScript Specification: https://tc39.es/ecma262/
- Types and Programming Languages (Pierce): https://www.cis.upenn.edu/~bcpierce/tapl/

---

*Document generated for Sprint 1 (TASK-000F)*  
*Historia: VELA-561 (US-00B)*  
*Last updated: 2025-11-30*
