# Vela Language Grammar Specification (EBNF)

**Version:** 0.1.0 (Phase 0 - Sprint 4)  
**Status:** Draft  
**Date:** 2025-11-30

## Notation

This specification uses Extended Backus-Naur Form (EBNF):

```
|   alternation
()  grouping
[]  option (0 or 1 times)
{}  repetition (0 to n times)
+   one or more times
*   zero or more times
```

## Lexical Grammar

### Tokens

```ebnf
(* Whitespace *)
WHITESPACE = " " | "\t" | "\r" | "\n" ;

(* Comments *)
LINE_COMMENT = "//" { ANY_CHAR - "\n" } "\n" ;
BLOCK_COMMENT = "/*" { ANY_CHAR } "*/" ;

(* Keywords *)
KEYWORD = "let" | "mut" | "const" | "fn" | "if" | "else" | "for" | "while" |
          "loop" | "break" | "continue" | "return" | "match" | "struct" |
          "enum" | "trait" | "impl" | "type" | "module" | "import" | "export" |
          "pub" | "priv" | "as" | "in" | "true" | "false" | "null" | "self" |
          "super" | "async" | "await" | "try" | "catch" | "throw" | "yield" |
          "widget" | "component" | "service" | "repository" | "controller" |
          "usecase" | "dto" | "entity" | "valueObject" | "model" | "factory" |
          "builder" | "strategy" | "observer" | "singleton" | "adapter" |
          "decorator" | "guard" | "middleware" | "interceptor" | "validator" |
          "store" | "provider" | "actor" | "pipe" | "task" | "helper" |
          "mapper" | "serializer" ;

(* Identifiers *)
IDENTIFIER = LETTER { LETTER | DIGIT | "_" } ;
LETTER = "a".."z" | "A".."Z" ;
DIGIT = "0".."9" ;

(* Literals *)
INTEGER = DIGIT+ ;
FLOAT = DIGIT+ "." DIGIT+ [ EXPONENT ] ;
EXPONENT = ("e" | "E") ["+" | "-"] DIGIT+ ;

STRING = "\"" { STRING_CHAR | ESCAPE_SEQUENCE } "\"" ;
STRING_CHAR = ANY_CHAR - ("\"" | "\\" | "\n") ;
ESCAPE_SEQUENCE = "\\" ("n" | "t" | "r" | "\"" | "\\" | "0" | "${") ;
STRING_INTERPOLATION = "${" EXPRESSION "}" ;

CHAR = "'" (CHAR_CHAR | ESCAPE_SEQUENCE) "'" ;
CHAR_CHAR = ANY_CHAR - ("'" | "\\") ;

(* Operators *)
OPERATOR = "+" | "-" | "*" | "/" | "%" | "**" |
           "==" | "!=" | "<" | ">" | "<=" | ">=" |
           "&&" | "||" | "!" |
           "&" | "|" | "^" | "~" | "<<" | ">>" |
           "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=" |
           "&=" | "|=" | "^=" | "<<=" | ">>=" |
           "??" | "?." | "?" | ":" |
           "=>" | "->" | "::" | ".." | "..=" | "..." ;

(* Delimiters *)
DELIMITER = "(" | ")" | "{" | "}" | "[" | "]" |
            ";" | "," | "." | "@" | "#" | "$" ;
```

## Syntactic Grammar

### Program Structure

```ebnf
Program = { TopLevelItem } ;

TopLevelItem = ModuleDecl
             | ImportDecl
             | ExportDecl
             | FunctionDecl
             | StructDecl
             | EnumDecl
             | TraitDecl
             | ImplBlock
             | TypeAlias
             | ConstDecl ;
```

### Module System

```ebnf
ModuleDecl = "module" IDENTIFIER "{" { TopLevelItem } "}" ;

ImportDecl = "import" ImportPath [ "as" IDENTIFIER ] ;
ImportPath = ImportPrefix IDENTIFIER { "::" IDENTIFIER } ;
ImportPrefix = "system:" | "package:" | "module:" | "library:" | 
               "extension:" | "assets:" ;

ExportDecl = "export" TopLevelItem ;
```

### Declarations

```ebnf
(* Function Declaration *)
FunctionDecl = [ "pub" ] [ "async" ] "fn" IDENTIFIER 
               [ GenericParams ] 
               "(" [ FunctionParams ] ")" 
               [ "->" Type ] 
               Block ;

FunctionParams = FunctionParam { "," FunctionParam } [ "," ] ;
FunctionParam = [ "mut" ] IDENTIFIER ":" Type ;

(* Variable Declaration *)
LetStmt = "let" [ "mut" ] Pattern [ ":" Type ] "=" Expression ";" ;
ConstDecl = "const" IDENTIFIER ":" Type "=" Expression ";" ;

(* Struct Declaration *)
StructDecl = [ "pub" ] "struct" IDENTIFIER [ GenericParams ] 
             ( "{" { StructField } "}" | ";" ) ;
StructField = [ "pub" ] IDENTIFIER ":" Type "," ;

(* Enum Declaration *)
EnumDecl = [ "pub" ] "enum" IDENTIFIER [ GenericParams ] 
           "{" { EnumVariant } "}" ;
EnumVariant = IDENTIFIER [ "(" { Type "," } ")" | "{" { StructField } "}" ] "," ;

(* Trait Declaration *)
TraitDecl = [ "pub" ] "trait" IDENTIFIER [ GenericParams ]
            [ ":" TraitBounds ]
            "{" { TraitItem } "}" ;
TraitItem = FunctionSignature | TypeAlias ;
FunctionSignature = "fn" IDENTIFIER [ GenericParams ] 
                    "(" [ FunctionParams ] ")" 
                    [ "->" Type ] ";" ;

(* Impl Block *)
ImplBlock = "impl" [ GenericParams ] 
            [ Trait "for" ] 
            Type 
            "{" { ImplItem } "}" ;
ImplItem = FunctionDecl | TypeAlias ;

(* Type Alias *)
TypeAlias = "type" IDENTIFIER [ GenericParams ] "=" Type ";" ;
```

### Statements

```ebnf
Statement = LetStmt
          | ExpressionStmt
          | ItemDecl
          | ";" ;

ExpressionStmt = Expression ";" ;

Block = "{" { Statement } [ Expression ] "}" ;
```

### Expressions

```ebnf
Expression = AssignmentExpr ;

AssignmentExpr = LogicalOrExpr [ AssignmentOp LogicalOrExpr ] ;
AssignmentOp = "=" | "+=" | "-=" | "*=" | "/=" | "%=" | "**=" |
               "&=" | "|=" | "^=" | "<<=" | ">>=" ;

LogicalOrExpr = LogicalAndExpr { "||" LogicalAndExpr } ;
LogicalAndExpr = NullCoalescingExpr { "&&" NullCoalescingExpr } ;
NullCoalescingExpr = EqualityExpr { "??" EqualityExpr } ;

EqualityExpr = ComparisonExpr { ("==" | "!=") ComparisonExpr } ;
ComparisonExpr = BitwiseOrExpr { ("<" | ">" | "<=" | ">=") BitwiseOrExpr } ;

BitwiseOrExpr = BitwiseXorExpr { "|" BitwiseXorExpr } ;
BitwiseXorExpr = BitwiseAndExpr { "^" BitwiseAndExpr } ;
BitwiseAndExpr = ShiftExpr { "&" ShiftExpr } ;

ShiftExpr = AdditiveExpr { ("<<" | ">>") AdditiveExpr } ;
AdditiveExpr = MultiplicativeExpr { ("+" | "-") MultiplicativeExpr } ;
MultiplicativeExpr = ExponentiationExpr { ("*" | "/" | "%") ExponentiationExpr } ;
ExponentiationExpr = UnaryExpr { "**" UnaryExpr } ;

UnaryExpr = ( "-" | "!" | "~" | "*" | "&" | "&mut" ) UnaryExpr
          | PostfixExpr ;

PostfixExpr = PrimaryExpr { PostfixOp } ;
PostfixOp = "(" [ Arguments ] ")"           (* Function call *)
          | "[" Expression "]"              (* Index *)
          | "." IDENTIFIER                  (* Field access *)
          | "?." IDENTIFIER                 (* Safe navigation *)
          | "?" ;                           (* Unwrap *)

PrimaryExpr = Literal
            | IDENTIFIER
            | "self"
            | "super"
            | Block
            | IfExpr
            | MatchExpr
            | LoopExpr
            | ForExpr
            | WhileExpr
            | TryExpr
            | AsyncExpr
            | LambdaExpr
            | ArrayExpr
            | TupleExpr
            | StructExpr
            | "(" Expression ")" ;
```

### Control Flow

```ebnf
(* If Expression *)
IfExpr = "if" Expression Block 
         [ "else" ( IfExpr | Block ) ] ;

(* Match Expression *)
MatchExpr = "match" Expression "{" { MatchArm } "}" ;
MatchArm = Pattern [ "if" Expression ] "=>" ( Expression "," | Block ) ;

(* Loop *)
LoopExpr = "loop" Block ;

(* For Loop *)
ForExpr = "for" Pattern "in" Expression Block ;

(* While Loop *)
WhileExpr = "while" Expression Block ;

(* Control Flow Keywords *)
BreakExpr = "break" [ Expression ] ;
ContinueExpr = "continue" ;
ReturnExpr = "return" [ Expression ] ;
```

### Async/Await

```ebnf
AsyncExpr = "async" Block ;
AwaitExpr = Expression "." "await" ;
```

### Error Handling

```ebnf
TryExpr = "try" Block 
          { "catch" Pattern Block } 
          [ "finally" Block ] ;

ThrowExpr = "throw" Expression ;
```

### Lambdas

```ebnf
LambdaExpr = "|" [ LambdaParams ] "|" ( Expression | Block ) ;
LambdaParams = LambdaParam { "," LambdaParam } ;
LambdaParam = Pattern [ ":" Type ] ;
```

### Literals

```ebnf
Literal = INTEGER
        | FLOAT
        | STRING
        | CHAR
        | "true"
        | "false"
        | "null" ;

ArrayExpr = "[" [ Expression { "," Expression } [ "," ] ] "]" ;

TupleExpr = "(" Expression "," [ Expression { "," Expression } ] [ "," ] ")" ;

StructExpr = PathExpr "{" [ StructField { "," StructField } [ "," ] ] "}" ;
StructFieldExpr = IDENTIFIER [ ":" Expression ] ;
```

### Patterns

```ebnf
Pattern = LiteralPattern
        | IdentifierPattern
        | WildcardPattern
        | TuplePattern
        | StructPattern
        | EnumPattern
        | OrPattern
        | RangePattern ;

LiteralPattern = Literal ;
IdentifierPattern = [ "mut" ] IDENTIFIER [ "@" Pattern ] ;
WildcardPattern = "_" ;
TuplePattern = "(" [ Pattern { "," Pattern } [ "," ] ] ")" ;
StructPattern = PathExpr "{" [ FieldPattern { "," FieldPattern } ] [ ".." ] "}" ;
FieldPattern = IDENTIFIER [ ":" Pattern ] ;
EnumPattern = PathExpr [ "(" Pattern { "," Pattern } ")" ] ;
OrPattern = Pattern { "|" Pattern } ;
RangePattern = Literal ".." [ "=" ] Literal ;
```

### Types

```ebnf
Type = PrimitiveType
     | ArrayType
     | TupleType
     | FunctionType
     | PathType
     | ReferenceType
     | OptionType
     | ResultType
     | GenericType ;

PrimitiveType = "i8" | "i16" | "i32" | "i64" | "i128" |
                "u8" | "u16" | "u32" | "u64" | "u128" |
                "f32" | "f64" |
                "bool" | "char" | "str" | "string" |
                "unit" | "never" ;

ArrayType = "[" Type ";" INTEGER "]" ;
TupleType = "(" [ Type { "," Type } [ "," ] ] ")" ;
FunctionType = "fn" "(" [ Type { "," Type } ] ")" [ "->" Type ] ;
PathType = IDENTIFIER { "::" IDENTIFIER } [ GenericArgs ] ;
ReferenceType = "&" [ "mut" ] Type ;
OptionType = Type "?" ;
ResultType = "Result" "<" Type "," Type ">" ;

GenericType = IDENTIFIER [ GenericArgs ] ;
GenericParams = "<" GenericParam { "," GenericParam } [ "," ] ">" ;
GenericParam = IDENTIFIER [ ":" TraitBounds ] [ "=" Type ] ;
GenericArgs = "<" Type { "," Type } [ "," ] ">" ;
TraitBounds = Trait { "+" Trait } ;
```

### Domain-Specific Keywords

```ebnf
(* Widget/Component *)
WidgetDecl = [ "pub" ] "widget" IDENTIFIER [ GenericParams ]
             "{" { WidgetItem } "}" ;
WidgetItem = StateDecl | MethodDecl ;
StateDecl = [ "mut" ] IDENTIFIER ":" Type [ "=" Expression ] ";" ;
MethodDecl = FunctionDecl ;

ComponentDecl = [ "pub" ] "component" IDENTIFIER [ GenericParams ]
                "{" { ComponentItem } "}" ;
ComponentItem = StateDecl | MethodDecl ;

(* Service Layer *)
ServiceDecl = [ "pub" ] "service" IDENTIFIER [ GenericParams ]
              "{" { ServiceMethod } "}" ;
ServiceMethod = FunctionDecl ;

RepositoryDecl = [ "pub" ] "repository" IDENTIFIER "<" Type ">"
                 "{" { RepositoryMethod } "}" ;
RepositoryMethod = FunctionDecl ;

(* Domain Models *)
EntityDecl = [ "pub" ] "entity" IDENTIFIER [ GenericParams ]
             "{" { EntityField } "}" ;
EntityField = [ "pub" ] IDENTIFIER ":" Type "," ;

DtoDecl = [ "pub" ] "dto" IDENTIFIER [ GenericParams ]
          "{" { DtoField } "}" ;
DtoField = [ "pub" ] IDENTIFIER ":" Type "," ;

ValueObjectDecl = [ "pub" ] "valueObject" IDENTIFIER [ GenericParams ]
                  "{" { ValueObjectField } "}" ;
ValueObjectField = [ "pub" ] IDENTIFIER ":" Type "," ;

(* Design Patterns *)
FactoryDecl = [ "pub" ] "factory" IDENTIFIER [ GenericParams ]
              "{" { FactoryMethod } "}" ;
FactoryMethod = "fn" "create" "(" [ FunctionParams ] ")" "->" Type Block ;

BuilderDecl = [ "pub" ] "builder" IDENTIFIER [ GenericParams ]
              "{" { BuilderMethod } "}" ;
BuilderMethod = FunctionDecl ;

StrategyDecl = [ "pub" ] "strategy" IDENTIFIER [ GenericParams ]
               "{" { StrategyMethod } "}" ;
StrategyMethod = FunctionDecl ;

(* Controller/Middleware *)
ControllerDecl = [ "pub" ] "controller" IDENTIFIER 
                 [ "@" STRING ]  (* Base path *)
                 "{" { ControllerMethod } "}" ;
ControllerMethod = [ HTTPDecorator ] FunctionDecl ;
HTTPDecorator = "@" ("get" | "post" | "put" | "delete" | "patch") 
                "(" STRING ")" ;

MiddlewareDecl = [ "pub" ] "middleware" IDENTIFIER
                 "{" { MiddlewareMethod } "}" ;
MiddlewareMethod = "fn" "handle" "(" FunctionParams ")" Block ;

GuardDecl = [ "pub" ] "guard" IDENTIFIER
            "{" { GuardMethod } "}" ;
GuardMethod = "fn" "canActivate" "(" FunctionParams ")" "->" "bool" Block ;
```

### Reactive System

```ebnf
(* Signals *)
SignalDecl = "let" IDENTIFIER "=" "Signal" "(" Expression ")" ";" ;
ComputedDecl = "let" IDENTIFIER "=" "Computed" "(" LambdaExpr ")" ";" ;
EffectDecl = "Effect" "(" LambdaExpr ")" ";" ;
WatchDecl = "Watch" "(" Expression "," LambdaExpr ")" ";" ;

(* Dependency Injection *)
InjectableDecorator = "@" "injectable" [ "(" ScopeArg ")" ] ;
ScopeArg = "Singleton" | "Transient" | "Scoped" ;
InjectDecorator = "@" "inject" ;
ContainerDecorator = "@" "container" ;
ProvidesDecorator = "@" "provides" ;

(* State Management *)
StoreDecl = "store" IDENTIFIER "<" Type ">" 
            "{" StateField "," ReducerMethods "}" ;
StateField = "state" ":" Type "," ;
ReducerMethods = { "fn" IDENTIFIER "(" FunctionParams ")" "->" Type Block } ;
DispatchExpr = "dispatch" "(" Expression ")" ;
```

### Attributes/Decorators

```ebnf
Attribute = "@" AttributePath [ "(" AttributeArgs ")" ] ;
AttributePath = IDENTIFIER { "::" IDENTIFIER } ;
AttributeArgs = Expression { "," Expression } [ "," ] ;
```

## Precedence Table

(From lowest to highest)

| Level | Operators | Associativity |
|-------|-----------|---------------|
| 1 | `=`, `+=`, `-=`, etc. | Right |
| 2 | `\|\|` | Left |
| 3 | `&&` | Left |
| 4 | `??` | Left |
| 5 | `==`, `!=` | Left |
| 6 | `<`, `>`, `<=`, `>=` | Left |
| 7 | `\|` | Left |
| 8 | `^` | Left |
| 9 | `&` | Left |
| 10 | `<<`, `>>` | Left |
| 11 | `+`, `-` | Left |
| 12 | `*`, `/`, `%` | Left |
| 13 | `**` | Right |
| 14 | `-`, `!`, `~`, `*`, `&` (unary) | Right |
| 15 | `()`, `[]`, `.`, `?.`, `?` (postfix) | Left |

## Notes

1. **Semicolons**: Required after statements except when the last expression in a block
2. **Trailing commas**: Allowed in lists (function params, struct fields, etc.)
3. **Unicode**: Full Unicode support in identifiers and strings
4. **String interpolation**: `"Hello, ${name}!"` (uses `${}` not `{}`)
5. **Optional chaining**: `obj?.field?.method()` returns `null` on first `null`
6. **Null coalescing**: `value ?? default` returns `default` if `value` is `null`
7. **Range patterns**: `1..=10` (inclusive), `1..10` (exclusive)

---

**TASK:** TASK-001  
**Historia:** VELA-566 (US-01)  
**Sprint:** Sprint 4 (Phase 0)  
**Status:** Completed ✅  
**Date:** 2025-11-30
