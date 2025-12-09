# TASK-035U: Implementar dispatch keyword

## 📋 Información General
- **Historia:** VELA-035 (EPIC-03D State Management)
- **Estado:** Completada ✅
- **Fecha:** 2025-12-09

## 🎯 Objetivo
Implementar el keyword `dispatch` para enviar acciones al store global de state management.

## 🔨 Implementación

### 1. Lexer (lexer.rs)
- Agregado `TokenKind::Dispatch`
- Mapping "dispatch" => `TokenKind::Dispatch`
- Actualizado test de keywords

### 2. AST (ast.rs)
- Agregado `Expression::Dispatch(DispatchExpression)`
- Definida struct `DispatchExpression` con campo `action: Box<Expression>`

### 3. Parser (parser.rs)
- Agregada regla de parsing para `dispatch(action_expr)`
- Sintaxis: `dispatch` `(` expression `)`

### 4. Semantic Analyzer (semantic.rs)
- Agregado `type_check_dispatch()` que valida la expresión de acción
- Retorna tipo `void` (side effect)

## ✅ Sintaxis Implementada

```vela
// Dispatch de acción simple
dispatch(IncrementCounter())

// Dispatch con acción que tiene payload
dispatch(SetCounterValue(42))

// Dispatch con acción custom
dispatch(UpdateUser(userId: "123", name: "John"))
```

## 🔗 Referencias
- **Jira:** [VELA-035U](https://velalang.atlassian.net/browse/VELA-035U)
- **Historia:** [VELA-035](https://velalang.atlassian.net/browse/VELA-035)
- **Dependencias:** TASK-035T (Action/Reducer types)