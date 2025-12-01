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
(* NOTE: Vela is PURE FUNCTIONAL - NO loops (for/while/loop), NO null, NO let/const/var *)
(* Variables are immutable by default (no keyword). Use 'state' for reactive mutability *)
KEYWORD = "state" | "fn" | "if" | "else" | "return" | "match" | "struct" |
          "enum" | "interface" | "class" | "extends" | "implements" | "override" |
          "abstract" | "this" | "super" | "constructor" | "overload" |
          "type" | "import" | "public" | "private" | "protected" |
          "as" | "show" | "hide" | "true" | "false" | "None" | "Some" |
          "async" | "await" | "try" | "catch" | "throw" | "finally" | "yield" |
          "computed" | "memo" | "effect" | "watch" | "mount" | "update" | "destroy" |
          "beforeUpdate" | "afterUpdate" | "StatefulWidget" | "StatelessWidget" |
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

TopLevelItem = ImportDecl
             | FunctionDecl
             | StructDecl
             | EnumDecl
             | ClassDecl
             | InterfaceDecl
             | ServiceDecl
             | TypeAlias
             | ImmutableDecl ;
```

### Module System

```ebnf
(* NOTE: NO 'export' keyword in Vela - use 'public' modifier *)
ImportDecl = "import" ImportPath [ ImportQualifiers ] [ "as" IDENTIFIER ] ;
ImportPath = "'" ImportPrefix IDENTIFIER "'" ;
ImportPrefix = "package:" | "lib:" | "system:" | "assets:" | "extension:" ;
ImportQualifiers = "show" "{" IDENTIFIER { "," IDENTIFIER } "}"
                 | "hide" "{" IDENTIFIER { "," IDENTIFIER } "}" ;
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
FunctionParam = IDENTIFIER ":" Type ;

(* Variable Declaration *)
(* NOTE: Immutable by default (no keyword). Use 'state' for reactive mutability *)
ImmutableDecl = IDENTIFIER ":" Type "=" Expression ";" ;
StateDecl = "state" IDENTIFIER ":" Type "=" Expression ";" ;

(* Struct Declaration *)
StructDecl = [ "pub" ] "struct" IDENTIFIER [ GenericParams ] 
             ( "{" { StructField } "}" | ";" ) ;
StructField = [ "pub" ] IDENTIFIER ":" Type "," ;

(* Enum Declaration *)
EnumDecl = [ "public" | "private" | "protected" ] "enum" IDENTIFIER [ GenericParams ] 
           "{" { EnumVariant } "}" ;
EnumVariant = IDENTIFIER [ "(" { Type "," } ")" | "{" { StructField } "}" ] "," ;

(* Class Declaration *)
ClassDecl = [ "public" | "private" | "protected" ] [ "abstract" ] "class" IDENTIFIER 
            [ GenericParams ] 
            [ "extends" Type ] 
            [ "implements" Type { "," Type } ]
            "{" { ClassMember } "}" ;
ClassMember = ConstructorDecl | MethodDecl | PropertyDecl ;
ConstructorDecl = "constructor" "(" [ FunctionParams ] ")" Block ;
MethodDecl = [ "override" | "overload" ] FunctionDecl ;
PropertyDecl = [ "state" ] IDENTIFIER ":" Type [ "=" Expression ] ";" ;

(* Interface Declaration *)
InterfaceDecl = [ "public" ] "interface" IDENTIFIER [ GenericParams ]
                [ "extends" Type { "," Type } ]
                "{" { InterfaceItem } "}" ;
InterfaceItem = FunctionSignature | PropertySignature ;
FunctionSignature = "fn" IDENTIFIER [ GenericParams ] 
                    "(" [ FunctionParams ] ")" 
                    [ "->" Type ] ";" ;
PropertySignature = IDENTIFIER ":" Type ";" ;

(* Type Alias *)
TypeAlias = "type" IDENTIFIER [ GenericParams ] "=" Type ";" ;
```

### Statements

```ebnf
Statement = ImmutableDecl
          | StateDecl
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
LogicalAndExpr = OptionCoalescingExpr { "&&" OptionCoalescingExpr } ;
(* NOTE: ?? operator for Option<T> coalescing, NOT null *)
OptionCoalescingExpr = EqualityExpr { "??" EqualityExpr } ;

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
          | "." IDENTIFIER                  (* Field/method access *)
          | "?." IDENTIFIER                 (* Safe navigation for Option<T> *)
          | "?" ;                           (* Unwrap Option<T> *)

(* NOTE: NO loop/for/while in Vela - use functional methods (.map, .filter, etc.) or recursion *)
PrimaryExpr = Literal
            | IDENTIFIER
            | "this"
            | "super"
            | Block
            | IfExpr
            | MatchExpr
            | TryExpr
            | AsyncExpr
            | LambdaExpr
            | ArrayExpr
            | TupleExpr
            | StructExpr
            | FunctionalMethodCall
            | "(" Expression ")" ;

(* Functional methods replace loops *)
FunctionalMethodCall = Expression "." FunctionalMethod "(" [ Arguments ] ")" ;
FunctionalMethod = "map" | "filter" | "reduce" | "forEach" | "flatMap" | "find" 
                 | "findIndex" | "every" | "some" | "take" | "drop" 
                 | "takeWhile" | "dropWhile" | "partition" | "groupBy" 
                 | "sortBy" | "chunk" | "zip" | "scan" | "distinct" | "reverse" ;
```

### Control Flow

```ebnf
(* If Expression - also returns value *)
IfExpr = "if" Expression Block 
         [ "else" ( IfExpr | Block ) ] ;

(* Match Expression - exhaustive pattern matching *)
MatchExpr = "match" Expression "{" { MatchArm } "}" ;
MatchArm = Pattern [ "if" Expression ] "=>" ( Expression "," | Block ) ;

(* Return - early exit from function *)
ReturnExpr = "return" [ Expression ] ;

(* NOTE: NO loop/for/while/break/continue in Vela *)
(* Use functional methods like .map(), .filter(), .forEach() instead *)
(* For infinite loops, use tail-call optimized recursion *)
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
        | "None"      (* Option<T> - NO null in Vela *)
        | "Some" "(" Expression ")" ;

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
IdentifierPattern = IDENTIFIER [ "@" Pattern ] ;
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

PrimitiveType = "Number"     (* 64-bit integer *)
              | "Float"      (* 64-bit float *)
              | "String"     (* UTF-8 string *)
              | "Bool"       (* boolean *)
              | "Char"       (* single Unicode character *)
              | "void"       (* no return value *)
              | "never" ;    (* never returns *)

ArrayType = "[" Type { "," Type } [ "," ] "]" ;
TupleType = "(" Type "," [ Type { "," Type } ] [ "," ] ")" ;
FunctionType = "fn" "(" [ Type { "," Type } ] ")" [ "->" Type ] ;
PathType = IDENTIFIER { "::" IDENTIFIER } [ GenericArgs ] ;
OptionType = "Option" "<" Type ">"    (* NO null - use Option<T> *)
           | Type "?" ;                (* Syntactic sugar for Option<T> *)
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
WidgetItem = PropertyDecl | MethodDecl | ComputedDecl | EffectDecl ;
MethodDecl = FunctionDecl ;

ComponentDecl = [ "public" ] "component" IDENTIFIER [ GenericParams ]
                "{" { ComponentItem } "}" ;
ComponentItem = PropertyDecl | MethodDecl | ComputedDecl | EffectDecl ;

(* Reactive declarations *)
ComputedDecl = "computed" IDENTIFIER ":" Type Block ;
EffectDecl = "effect" Block ;
WatchDecl = "watch" "(" Expression ")" Block ;

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
5. **Optional chaining**: `obj?.field?.method()` returns `None` on first `None` (for Option<T>)
6. **Option coalescing**: `value ?? default` returns `default` if `value` is `None`
7. **Range patterns**: `1..=10` (inclusive), `1..10` (exclusive)
8. **NO loops**: Vela is PURE FUNCTIONAL - use `.map()`, `.filter()`, `.forEach()`, `.reduce()` instead
9. **Immutability**: Variables immutable by default (no keyword). Use `state` for reactive mutability
10. **NO null**: Use `Option<T>` with `Some(value)` or `None` instead of null/undefined/nil

---

**TASK:** TASK-001  
**Historia:** VELA-566 (US-01)  
**Sprint:** Sprint 4 (Phase 0)  
**Status:** Completed ✅  
**Date:** 2025-11-30
