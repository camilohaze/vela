# Vela Language Specification

## Version 1.0.0

**Fecha:** 2025-12-14
**Estado:** Estable

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Lexical Structure](#2-lexical-structure)
3. [Syntax](#3-syntax)
4. [Types](#4-types)
5. [Expressions](#5-expressions)
6. [Statements](#6-statements)
7. [Declarations](#7-declarations)
8. [Modules and Imports](#8-modules-and-imports)
9. [Memory Model](#9-memory-model)
10. [Concurrency Model](#10-concurrency-model)
11. [Standard Library](#11-standard-library)
12. [Extensions and Decorators](#12-extensions-and-decorators)
13. [Implementation Notes](#13-implementation-notes)

---

## 1. Introduction

Vela is a modern, functional-first programming language designed for building reactive, concurrent, and user interface applications. Vela emphasizes:

- **Functional purity**: Immutable data by default, pure functions
- **Reactivity**: Built-in signal system for reactive programming
- **Concurrency**: Actor-based concurrency model
- **Type safety**: Static typing with type inference
- **Multi-platform**: Compiles to VM bytecode, WebAssembly, and native code

### 1.1 Design Principles

1. **Functional First**: All constructs encourage functional programming patterns
2. **Zero-Cost Abstractions**: High-level constructs compile to efficient code
3. **Type Safety**: Prevent runtime errors through static analysis
4. **Composability**: Easy composition of functions and data structures
5. **Performance**: Optimized for both development and runtime performance

### 1.2 Execution Model

Vela programs execute in one of three backends:

- **VelaVM**: Stack-based virtual machine for interpreted execution
- **VelaWeb**: WebAssembly compilation for browser execution
- **VelaNative**: LLVM compilation for native performance

---

## 2. Lexical Structure

### 2.1 Character Set

Vela source code is UTF-8 encoded. The language supports Unicode identifiers and string literals.

### 2.2 Tokens

```
token ::= keyword | identifier | literal | operator | punctuation
```

#### 2.2.1 Keywords

**Primitive Types:**
```
Number, Float, String, Bool, void, never
```

**Control Flow:**
```
if, else, match
```

**Declarations:**
```
fn, let, state, type, enum, struct, class, interface, abstract, extends, implements
```

**Modifiers:**
```
public, private, protected, static, const, async, override, abstract
```

**Modules:**
```
import, module, package, library, extension, assets
```

**Error Handling:**
```
try, catch, throw
```

**Reactive:**
```
computed, memo, effect, watch
```

**UI:**
```
component, widget, StatefulWidget, StatelessWidget
```

**Patterns:**
```
factory, builder, strategy, observer, singleton, adapter, decorator, pipe, guard, middleware, interceptor, validator, service, repository, controller, usecase, entity, dto, valueObject, model, helper, mapper, serializer, provider, store, actor, task
```

**Special:**
```
this, super, constructor, return, yield, await, break, continue, in, as, is
```

#### 2.2.2 Identifiers

```
identifier ::= (letter | '_') (letter | digit | '_')*
letter ::= 'a'..'z' | 'A'..'Z' | unicode_letter
digit ::= '0'..'9'
```

#### 2.2.3 Literals

**Number Literals:**
```
number_literal ::= decimal_literal | hex_literal | binary_literal
decimal_literal ::= digit+ ('.' digit+)? ('e' ('+'|'-')? digit+)?
hex_literal ::= '0x' hex_digit+
binary_literal ::= '0b' ('0'|'1')+
```

**String Literals:**
```
string_literal ::= '"' (character | escape_sequence)* '"' | '"""' multiline_content '"""'
escape_sequence ::= '\\' ('n'|'t'|'r'|'\\'|'"'|'\${')
```

**Boolean Literals:**
```
boolean_literal ::= 'true' | 'false'
```

**Array Literals:**
```
array_literal ::= '[' expression_list? ']'
```

**Object Literals:**
```
object_literal ::= '{' (identifier ':' expression)* '}'
```

#### 2.2.4 Operators

**Arithmetic:** `+`, `-`, `*`, `/`, `%`, `**`
**Comparison:** `==`, `!=`, `<`, `<=`, `>`, `>=`
**Logical:** `&&`, `||`, `!`
**Bitwise:** `&`, `|`, `^`, `<<`, `>>`, `>>>`
**Assignment:** `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`
**Range:** `..`, `..=`
**Access:** `.`, `?.`, `?:`
**Other:** `=>`, `::`, `?`, `!`

#### 2.2.5 Punctuation

```
punctuation ::= '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | ':' | '::' | '.' | '..' | '...' | '@' | '#' | '$' | '?' | '!'
```

### 2.3 Comments

```
// Line comment

/*
Block comment
*/

/**
 * Documentation comment
 * @param param description
 * @return description
 */
```

---

## 3. Syntax

### 3.1 Grammar

The Vela grammar is defined using Extended Backus-Naur Form (EBNF).

#### 3.1.1 Program Structure

```
program ::= module_declaration* top_level_declaration*

module_declaration ::= 'module' identifier '{' module_body '}'

top_level_declaration ::= function_declaration
                        | type_declaration
                        | variable_declaration
                        | import_declaration
                        | export_declaration
```

#### 3.1.2 Declarations

```
function_declaration ::= 'fn' identifier '(' parameter_list? ')' ('->' type)? block
                        | 'async' 'fn' identifier '(' parameter_list? ')' ('->' type)? block

parameter_list ::= parameter (',' parameter)*

parameter ::= identifier ':' type

type_declaration ::= 'type' identifier '=' type_expression
                    | 'enum' identifier '{' enum_variant* '}'
                    | 'struct' identifier '{' field_declaration* '}'
                    | 'class' identifier ('extends' type)? ('implements' type_list)? '{' class_body '}'
                    | 'interface' identifier '{' interface_member* '}'

variable_declaration ::= identifier ':' type ('=' expression)?
                        | 'state' identifier ':' type ('=' expression)?

field_declaration ::= identifier ':' type

enum_variant ::= identifier ('(' type_list ')')?

class_body ::= (field_declaration | method_declaration | constructor_declaration)*

interface_member ::= method_signature

method_declaration ::= ('override')? 'fn' identifier '(' parameter_list? ')' ('->' type)? block

constructor_declaration ::= 'constructor' '(' parameter_list? ')' block

method_signature ::= 'fn' identifier '(' parameter_list? ')' ('->' type)?
```

#### 3.1.3 Statements

```
statement ::= expression_statement
             | variable_declaration_statement
             | assignment_statement
             | if_statement
             | match_statement
             | loop_statement
             | return_statement
             | throw_statement
             | try_statement
             | block_statement

expression_statement ::= expression ';'

assignment_statement ::= expression '=' expression ';'

if_statement ::= 'if' expression block ('else' block)?

match_statement ::= 'match' expression '{' match_arm* '}'

match_arm ::= pattern '=>' (expression | block)

loop_statement ::= 'for' pattern 'in' expression block
                  | 'while' expression block

return_statement ::= 'return' expression? ';'

throw_statement ::= 'throw' expression ';'

try_statement ::= 'try' block catch_clause* finally_clause?

catch_clause ::= 'catch' '(' identifier ':' type ')' block

finally_clause ::= 'finally' block

block_statement ::= '{' statement* '}'
```

#### 3.1.4 Expressions

```
expression ::= primary_expression
              | unary_expression
              | binary_expression
              | ternary_expression
              | call_expression
              | member_expression
              | index_expression
              | lambda_expression
              | array_expression
              | object_expression
              | if_expression
              | match_expression

primary_expression ::= identifier
                      | literal
                      | '(' expression ')'

unary_expression ::= ('+'|'-'|'!'|'~') expression

binary_expression ::= expression binary_operator expression

ternary_expression ::= expression '?' expression ':' expression

call_expression ::= expression '(' argument_list? ')'

member_expression ::= expression '.' identifier

index_expression ::= expression '[' expression ']'

lambda_expression ::= '(' parameter_list? ')' '=>' (expression | block)

array_expression ::= '[' expression_list? ']'

object_expression ::= '{' (identifier ':' expression)* '}'

if_expression ::= 'if' expression block 'else' block

match_expression ::= 'match' expression '{' match_arm* '}'

argument_list ::= expression (',' expression)*

expression_list ::= expression (',' expression)*
```

#### 3.1.5 Patterns

```
pattern ::= literal_pattern
           | identifier_pattern
           | wildcard_pattern
           | tuple_pattern
           | struct_pattern
           | enum_pattern
           | array_pattern
           | or_pattern
           | guard_pattern

literal_pattern ::= literal

identifier_pattern ::= identifier

wildcard_pattern ::= '_'

tuple_pattern ::= '(' pattern_list? ')'

struct_pattern ::= identifier '{' (identifier ':' pattern)* '}'

enum_pattern ::= identifier '::' identifier ('(' pattern_list? ')')?

array_pattern ::= '[' pattern_list? ']'

or_pattern ::= pattern '|' pattern

guard_pattern ::= pattern 'if' expression

pattern_list ::= pattern (',' pattern)*
```

#### 3.1.6 Types

```
type ::= primitive_type
        | identifier
        | generic_type
        | function_type
        | tuple_type
        | array_type
        | option_type
        | result_type

primitive_type ::= 'Number' | 'Float' | 'String' | 'Bool' | 'void' | 'never'

generic_type ::= identifier '<' type_list '>'

function_type ::= '(' type_list? ')' '->' type

tuple_type ::= '(' type_list ')'

array_type ::= '[' type ']'

option_type ::= 'Option' '<' type '>'

result_type ::= 'Result' '<' type ',' type '>'

type_list ::= type (',' type)*
```

### 3.2 Operator Precedence

From highest to lowest precedence:

1. Member access: `.`, `?.`
2. Function call: `(...)`
3. Array access: `[...]`
4. Unary: `+`, `-`, `!`, `~`
5. Power: `**`
6. Multiplicative: `*`, `/`, `%`
7. Additive: `+`, `-`
8. Shift: `<<`, `>>`, `>>>`
9. Relational: `<`, `<=`, `>`, `>=`
10. Equality: `==`, `!=`
11. Bitwise AND: `&`
12. Bitwise XOR: `^`
13. Bitwise OR: `|`
14. Logical AND: `&&`
15. Logical OR: `||`
16. Ternary: `? :`
17. Assignment: `=`, `+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`, `<<=`, `>>=`
18. Range: `..`, `..=`

---

## 4. Types

### 4.1 Type System Overview

Vela has a static type system with type inference based on Hindley-Milner algorithm. Types are inferred where possible, but explicit type annotations are required for function parameters and public APIs.

### 4.2 Primitive Types

- **Number**: 64-bit signed integer
- **Float**: 64-bit IEEE 754 floating point
- **String**: UTF-8 encoded string
- **Bool**: Boolean value (`true` or `false`)
- **void**: Unit type (no value)
- **never**: Bottom type (never returns)

### 4.3 Composite Types

#### 4.3.1 Arrays

```
type ::= '[' element_type ']'
```

Arrays are immutable by default and support functional operations.

#### 4.3.2 Tuples

```
type ::= '(' type_list ')'
```

Tuples are heterogeneous sequences of fixed size.

#### 4.3.3 Option Types

```
type ::= 'Option' '<' T '>'
```

Represents optional values. Constructors: `Some(value)` or `None`.

#### 4.3.4 Result Types

```
type ::= 'Result' '<' T ',' E '>'
```

Represents computation results. Constructors: `Ok(value)` or `Err(error)`.

### 4.4 User-Defined Types

#### 4.4.1 Type Aliases

```
type_alias ::= 'type' identifier '=' type_expression
```

#### 4.4.2 Structs

```
struct_declaration ::= 'struct' identifier '{' field_declaration* '}'
```

#### 4.4.3 Enums

```
enum_declaration ::= 'enum' identifier '{' enum_variant* '}'
enum_variant ::= identifier ('(' type_list ')')?
```

#### 4.4.4 Classes

```
class_declaration ::= 'class' identifier ('extends' type)? ('implements' type_list)? '{' class_body '}'
```

#### 4.4.5 Interfaces

```
interface_declaration ::= 'interface' identifier '{' method_signature* '}'
```

### 4.5 Generic Types

Vela supports parametric polymorphism:

```
generic_type ::= identifier '<' type_parameter_list '>'

type_parameter ::= identifier (':' type)?
```

### 4.6 Type Inference Rules

1. **Variable declarations**: Type inferred from initializer
2. **Function parameters**: Must be explicitly typed
3. **Function return**: Inferred from return expressions, or explicitly typed
4. **Generic instantiation**: Types inferred from usage context

### 4.7 Subtyping

Vela has structural subtyping for interfaces and nominal subtyping for classes.

---

## 5. Expressions

### 5.1 Evaluation Semantics

Expressions are evaluated left-to-right, with short-circuiting for logical operators.

### 5.2 Primary Expressions

- **Literals**: Evaluate to their values
- **Identifiers**: Look up in current scope
- **Parenthesized**: Evaluate inner expression

### 5.3 Function Calls

```
call_expression ::= expression '(' argument_list? ')'
```

Arguments are evaluated left-to-right and passed by value.

### 5.4 Member Access

```
member_expression ::= expression '.' identifier
```

Accesses fields or methods. For optionals, use `?.` for safe access.

### 5.5 Array Access

```
index_expression ::= expression '[' expression ']'
```

Index must be a Number. Bounds checking is performed at runtime.

### 5.6 Lambda Expressions

```
lambda_expression ::= '(' parameter_list? ')' '=>' (expression | block)
```

Creates anonymous functions. Captures variables by reference.

### 5.7 Control Flow Expressions

#### 5.7.1 If Expressions

```
if_expression ::= 'if' condition block 'else' block
```

Both branches must have compatible types.

#### 5.7.2 Match Expressions

```
match_expression ::= 'match' scrutinee '{' match_arm* '}'
match_arm ::= pattern '=>' expression
```

Exhaustive matching required. Patterns are tested in order.

---

## 6. Statements

### 6.1 Expression Statements

Expressions followed by semicolon are executed for their side effects.

### 6.2 Declaration Statements

Introduce new bindings in the current scope.

### 6.3 Assignment Statements

```
assignment_statement ::= lvalue '=' expression ';'
```

Lvalue must be mutable (declared with `state` or mutable field).

### 6.4 Control Flow Statements

#### 6.4.1 If Statements

```
if_statement ::= 'if' condition block ('else' block)?
```

#### 6.4.2 Match Statements

```
match_statement ::= 'match' scrutinee '{' match_arm* '}'
```

Same semantics as match expressions.

#### 6.4.3 Loops

Vela does not have imperative loops. Use functional constructs:

```vela
// Instead of for loops
list.forEach(item => { /* ... */ })

// Instead of while loops
fn repeatWhile(condition: () -> Bool, action: () -> void) -> void {
  if condition() {
    action()
    repeatWhile(condition, action)
  }
}
```

#### 6.4.4 Return Statements

```
return_statement ::= 'return' expression? ';'
```

Returns control and optional value to caller.

#### 6.4.5 Throw Statements

```
throw_statement ::= 'throw' expression ';'
```

Throws an exception. Use `Result<T,E>` for typed errors.

### 6.5 Exception Handling

```
try_statement ::= 'try' block catch_clause* finally_clause?
catch_clause ::= 'catch' '(' identifier ':' type ')' block
finally_clause ::= 'finally' block
```

---

## 7. Declarations

### 7.1 Variable Declarations

```
variable_declaration ::= identifier ':' type ('=' expression)?
                        | 'state' identifier ':' type ('=' expression)?
```

Variables are immutable by default. Use `state` for reactive mutability.

### 7.2 Function Declarations

```
function_declaration ::= 'fn' identifier '(' parameter_list? ')' ('->' type)? block
                        | 'async' 'fn' identifier '(' parameter_list? ')' ('->' type)? block
```

Functions are first-class values.

### 7.3 Type Declarations

See section 4.4 for type declaration syntax.

### 7.4 Module Declarations

```
module_declaration ::= '@module' '(' module_config ')' 'module' identifier '{' module_body '}'
```

Modules organize code and dependencies.

---

## 8. Modules and Imports

### 8.1 Import System

Vela uses a prefixed import system:

```
import ::= 'import' import_path (('show' | 'hide') identifier_list)?

import_path ::= string_literal
               | prefixed_path

prefixed_path ::= prefix '::' path_segments

prefix ::= 'system' | 'package' | 'module' | 'library' | 'extension' | 'assets'
```

### 8.2 Module Resolution

Modules are resolved in this order:

1. Built-in modules (`system:*`)
2. Project modules (`module:*`)
3. External packages (`package:*`)
4. Internal libraries (`library:*`)
5. Extensions (`extension:*`)
6. Assets (`assets:*`)

### 8.3 Visibility

Declarations are private by default. Use `public` modifier for exports.

---

## 9. Memory Model

### 9.1 Ownership

Vela uses Automatic Reference Counting (ARC) for memory management.

### 9.2 Object Lifetime

Objects are deallocated when their reference count reaches zero.

### 9.3 Cycle Detection

ARC is augmented with cycle detection for reference cycles.

### 9.4 Signals and Reactivity

Signals maintain dependency graphs and propagate changes automatically.

---

## 10. Concurrency Model

### 10.1 Actors

Vela uses actor-based concurrency:

```
actor_declaration ::= 'actor' identifier '{' actor_body '}'

actor_body ::= (field_declaration | method_declaration | message_handler)*

message_handler ::= 'handle' identifier '(' parameter_list? ')' block
```

### 10.2 Message Passing

Actors communicate via asynchronous message passing.

### 10.3 Signal Propagation

Signals propagate changes across actor boundaries.

---

## 11. Standard Library

### 11.1 Core APIs

The standard library provides essential functionality:

- **Collections**: Array, Map, Set with functional operations
- **Option/Result**: Type-safe error handling
- **Strings**: UTF-8 string manipulation
- **Math**: Mathematical functions and constants
- **IO**: File and network operations
- **Time**: Date/time handling
- **JSON**: JSON serialization/deserialization
- **HTTP**: HTTP client and server APIs
- **Reactive**: Signal, computed, effect APIs
- **UI**: Widget system for user interfaces

### 11.2 Contracts

All stdlib APIs have formal contracts with preconditions, postconditions, and invariants.

---

## 12. Extensions and Decorators

### 12.1 Decorator Syntax

```
decorator ::= '@' identifier ('(' argument_list? ')')?
decorated_declaration ::= decorator* declaration
```

### 12.2 Built-in Decorators

- **Dependency Injection**: `@injectable`, `@inject`, `@provides`
- **HTTP**: `@get`, `@post`, `@put`, `@delete`
- **Validation**: `@required`, `@email`, `@min`, `@max`
- **UI**: `@component`, `@widget`
- **Lifecycle**: `@mount`, `@update`, `@destroy`
- **Reactive**: `@computed`, `@memo`, `@effect`

### 12.3 Custom Decorators

Users can define custom decorators using the extension system.

---

## 13. Implementation Notes

### 13.1 Compilation Pipeline

1. **Lexing**: Source → tokens
2. **Parsing**: Tokens → AST
3. **Type Checking**: AST → typed AST
4. **IR Generation**: Typed AST → Vela IR
5. **Optimization**: Vela IR → optimized IR
6. **Code Generation**: IR → target bytecode/native/WebAssembly

### 13.2 Runtime Requirements

- **VelaVM**: Stack-based execution with garbage collection
- **WebAssembly**: Browser-compatible compilation
- **Native**: LLVM-based compilation with system libraries

### 13.3 Performance Characteristics

- **Functional operations**: O(1) amortized for persistent data structures
- **Signal propagation**: O(number of dependencies) for updates
- **Actor messaging**: Asynchronous with bounded queues
- **Memory**: ARC with cycle detection, predictable pauses

---

This specification is the authoritative reference for the Vela programming language. All implementations must conform to these rules.