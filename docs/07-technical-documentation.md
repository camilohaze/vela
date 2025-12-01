# 7. Documentación Técnica - Estructura y Guías

## 7.1 Estructura de Documentación

### 7.1.1 Jerarquía de Documentos

```
vela-docs/
├── README.md
├── GETTING_STARTED.md
├── CHANGELOG.md
├── CONTRIBUTING.md
├── CODE_OF_CONDUCT.md
│
├── language-specification/
│   ├── 01-grammar-syntax.md
│   ├── 02-type-system.md
│   ├── 03-modules-imports.md
│   ├── 04-functions-closures.md
│   ├── 05-classes-oop.md
│   ├── 06-interfaces-protocols.md
│   ├── 07-enums-adts.md
│   ├── 08-generics.md
│   ├── 09-error-handling.md
│   └── 10-memory-management.md
│
├── core-concepts/
│   ├── signals-reactivity.md
│   ├── state-management.md
│   ├── effects-side-effects.md
│   ├── computed-values.md
│   └── watchers.md
│
├── concurrency/
│   ├── actors-model.md
│   ├── async-await.md
│   ├── workers-threads.md
│   ├── channels-messaging.md
│   └── structured-concurrency.md
│
├── ui-framework/
│   ├── widgets-overview.md
│   ├── layout-system.md
│   ├── styling-theming.md
│   ├── animations.md
│   ├── gestures-input.md
│   ├── navigation-routing.md
│   └── platform-integration.md
│
├── standard-library/
│   ├── collections.md
│   ├── io-filesystem.md
│   ├── networking-http.md
│   ├── serialization-json.md
│   ├── testing.md
│   └── utilities.md
│
├── compiler-internals/
│   ├── architecture-overview.md
│   ├── lexer-parser.md
│   ├── type-checking.md
│   ├── ir-generation.md
│   ├── optimization-passes.md
│   ├── code-generation.md
│   └── debugging-support.md
│
├── runtime/
│   ├── velavm-architecture.md
│   ├── memory-model.md
│   ├── gc-arc-hybrid.md
│   ├── reactive-scheduler.md
│   ├── actor-runtime.md
│   └── performance-tuning.md
│
├── tooling/
│   ├── cli-reference.md
│   ├── package-manager.md
│   ├── lsp-server.md
│   ├── formatter.md
│   ├── linter.md
│   └── debugger.md
│
├── guides/
│   ├── building-first-app.md
│   ├── state-management-patterns.md
│   ├── async-programming.md
│   ├── testing-best-practices.md
│   ├── performance-optimization.md
│   ├── deployment.md
│   ├── migration-from-x.md
│   └── troubleshooting.md
│
├── tutorials/
│   ├── todo-app.md
│   ├── chat-application.md
│   ├── real-time-dashboard.md
│   ├── mobile-app.md
│   └── web-application.md
│
├── api-reference/
│   ├── core/
│   ├── reactive/
│   ├── async/
│   ├── ui/
│   ├── io/
│   └── net/
│
└── architecture-decisions/
    ├── ADR-001-reactive-system.md
    ├── ADR-002-actor-model.md
    ├── ADR-003-type-system.md
    ├── ADR-004-memory-management.md
    └── ...
```

---

## 7.2 Plantillas de Documentación

### 7.2.1 Plantilla: Guía de Concepto

```markdown
# [Nombre del Concepto]

## Resumen Ejecutivo
[1-2 párrafos explicando qué es y por qué importa]

## ¿Cuándo usar esto?
[Casos de uso claros]

## Sintaxis Básica
[Código de ejemplo simple]

## Conceptos Clave
### [Concepto 1]
[Explicación detallada]

### [Concepto 2]
[Explicación detallada]

## Ejemplos Completos
### Ejemplo 1: [Título]
[Código completo con explicaciones]

### Ejemplo 2: [Título]
[Código completo con explicaciones]

## Patrones Comunes
[Mejores prácticas y patrones idiomáticos]

## Antipatrones
[Qué NO hacer y por qué]

## Performance Considerations
[Implicaciones de rendimiento]

## Troubleshooting
[Problemas comunes y soluciones]

## Ver También
[Links a documentación relacionada]

## Referencias
[Links externos, papers, etc.]
```

---

### 7.2.2 Plantilla: API Reference

```markdown
# [API Name]

**Module**: `vela.module.name`  
**Since**: Version X.X  
**Status**: Stable | Experimental | Deprecated

## Description
[Descripción de 2-3 párrafos de qué hace esta API]

## Syntax

\`\`\`vela
fn functionName<T>(
  param1: Type1,
  param2: Type2
): ReturnType;
\`\`\`

## Parameters

### `param1`
- **Type**: `Type1`
- **Required**: Yes | No
- **Default**: `value` (if optional)
- **Description**: [Qué hace este parámetro]

### `param2`
- **Type**: `Type2`
- **Required**: Yes | No
- **Description**: [Qué hace este parámetro]

## Return Value

**Type**: `ReturnType`

[Descripción de qué retorna]

## Exceptions

- `ExceptionType1`: [Cuándo se lanza]
- `ExceptionType2`: [Cuándo se lanza]

## Examples

### Basic Usage
\`\`\`vela
[código de ejemplo básico]
\`\`\`

### Advanced Usage
\`\`\`vela
[código de ejemplo avanzado]
\`\`\`

## Notes
[Notas importantes, edge cases, etc.]

## See Also
- [Related API 1]
- [Related API 2]
```

---

### 7.2.3 Plantilla: Tutorial

```markdown
# Tutorial: [Título]

**Level**: Beginner | Intermediate | Advanced  
**Duration**: ~X minutes  
**Prerequisites**: [Lista de conocimientos previos]

## What You'll Build
[Descripción y screenshot/demo del resultado final]

## Learning Objectives
- [Objetivo 1]
- [Objetivo 2]
- [Objetivo 3]

## Prerequisites
- [Requisito 1]
- [Requisito 2]

## Step 1: [Título]
[Explicación de qué haremos]

\`\`\`vela
[código]
\`\`\`

[Explicación línea por línea si es necesario]

## Step 2: [Título]
[Continuar con pasos numerados]

## Testing
[Cómo probar que funciona]

## Next Steps
[Qué aprender después]

## Complete Code
[Código completo al final]

## Troubleshooting
[Problemas comunes en este tutorial]
```

---

### 7.2.4 Plantilla: Architecture Decision Record (ADR)

```markdown
# ADR-XXX: [Título de la Decisión]

**Date**: YYYY-MM-DD  
**Status**: Proposed | Accepted | Deprecated | Superseded  
**Deciders**: [Lista de personas]  
**Technical Story**: [Link a issue/ticket]

## Context
[Descripción del problema y contexto técnico]

## Decision Drivers
- [Factor 1 que influye en la decisión]
- [Factor 2]
- [Factor 3]

## Considered Options
1. [Opción 1]
2. [Opción 2]
3. [Opción 3]

## Decision Outcome

**Chosen option**: [Opción elegida]

**Reasoning**: [Por qué esta opción es la mejor]

## Pros and Cons of the Options

### [Opción 1]
**Pros**:
- [Pro 1]
- [Pro 2]

**Cons**:
- [Con 1]
- [Con 2]

### [Opción 2]
[Repetir para cada opción]

## Implementation Notes
[Notas técnicas sobre cómo implementar]

## Consequences

**Positive**:
- [Consecuencia positiva 1]
- [Consecuencia positiva 2]

**Negative**:
- [Consecuencia negativa 1]
- [Consecuencia negativa 2]

**Risks**:
- [Riesgo 1 y mitigación]

## References
- [Link 1]
- [Link 2]
```

---

## 7.3 Ejemplos de Documentación Completa

### 7.3.1 Getting Started Guide

```markdown
# Getting Started with Vela

Welcome to Vela! This guide will help you set up your development environment and build your first Vela application.

## Installation

### Prerequisites
- Operating System: Windows 10+, macOS 10.15+, or Linux
- Disk Space: ~500MB
- Internet connection

### Install Vela CLI

**macOS/Linux**:
\`\`\`bash
curl -fsSL https://vela-lang.org/install.sh | sh
\`\`\`

**Windows**:
\`\`\`powershell
irm https://vela-lang.org/install.ps1 | iex
\`\`\`

### Verify Installation
\`\`\`bash
vela --version
\`\`\`

Expected output: `Vela 1.0.0`

## Your First Vela Program

### 1. Create a New Project
\`\`\`bash
vela create hello-world
cd hello-world
\`\`\`

This creates:
\`\`\`
hello-world/
├── vela.yaml          # Project manifest
├── src/
│   └── main.vela      # Entry point
└── test/
    └── main_test.vela # Tests
\`\`\`

### 2. Write Code

Edit `src/main.vela`:
\`\`\`vela
module main;

fn main(): void {
  print("Hello, Vela!");
}
\`\`\`

### 3. Run Your Program
\`\`\`bash
vela run
\`\`\`

Output: `Hello, Vela!`

## Building a Counter App

Let's build something interactive:

\`\`\`vela
module main;

import vela.ui.{Container, Column, Text, Button};
import vela.reactive.{signal};

fn Counter(): Widget {
  let count = signal(0);
  
  return Container {
    padding: EdgeInsets.all(20),
    
    Column {
      spacing: 16,
      
      Text("Count: ${count.value}"),
      
      Button {
        label: "Increment",
        onClick: () => state { count.value += 1; }
      }
    }
  };
}

fn main(): void {
  App.run(Counter());
}
\`\`\`

Run with:
\`\`\`bash
vela run --target=desktop
\`\`\`

## Next Steps

- [Language Tour](./language-tour.md)
- [State Management Guide](./guides/state-management.md)
- [Building a Todo App Tutorial](./tutorials/todo-app.md)

## Getting Help

- [Documentation](https://docs.vela-lang.org)
- [Community Discord](https://discord.gg/vela)
- [Stack Overflow](https://stackoverflow.com/questions/tagged/vela)
- [GitHub Issues](https://github.com/vela-lang/vela/issues)
```

---

### 7.3.2 API Reference Example

```markdown
# signal<T>

**Module**: `vela.reactive`  
**Since**: Version 1.0  
**Status**: Stable

## Description

Creates a reactive signal that tracks and propagates changes to dependent computations and effects. Signals are the foundation of Vela's reactive system.

When you read a signal's value inside a `computed()` or `effect()`, that signal is automatically tracked as a dependency. When the signal's value changes, all dependent computations are invalidated and effects are re-executed.

## Syntax

\`\`\`vela
fn signal<T>(initialValue: T): Signal<T>;
\`\`\`

## Parameters

### `initialValue`
- **Type**: `T`
- **Required**: Yes
- **Description**: The initial value of the signal. Can be of any type.

## Return Value

**Type**: `Signal<T>`

Returns a Signal object with the following properties and methods:

- `value: T` - Getter/setter for the signal's value
- `subscribe((T) => void): Subscription` - Subscribe to changes
- `update((T) => T): void` - Update value based on previous value

## Examples

### Basic Usage

\`\`\`vela
let count = signal(0);

// Reading
print(count.value);  // 0

// Writing
state {
  count.value = 10;
}
\`\`\`

### With Effect

\`\`\`vela
let name = signal("Alice");

effect(() => {
  print("Hello, ${name.value}!");  // Tracks 'name' automatically
});

// Changing name triggers effect
state { name.value = "Bob"; }  // Prints: "Hello, Bob!"
\`\`\`

### Update Method

\`\`\`vela
let count = signal(0);

// Update based on previous value
count.update((prev) => prev + 1);
\`\`\`

### With Computed

\`\`\`vela
let firstName = signal("John");
let lastName = signal("Doe");

let fullName = computed(() => {
  return "${firstName.value} ${lastName.value}";
});

print(fullName.value);  // "John Doe"

state { firstName.value = "Jane"; }
print(fullName.value);  // "Jane Doe"
\`\`\`

## Type Safety

Signals are fully type-safe:

\`\`\`vela
let count: Signal<Int> = signal(0);

count.value = 10;      // OK
count.value = "hello"; // Compile error: Type mismatch
\`\`\`

## Performance Notes

- Signal reads are O(1)
- Signal writes trigger dependent updates in O(n) where n is number of direct dependents
- Use `batch()` to group multiple signal updates and trigger only one propagation

## Thread Safety

Signals are NOT thread-safe by default. If you need to update signals from multiple threads, use actors or channels for communication.

## See Also

- [computed()](./computed.md) - Derived values
- [effect()](./effect.md) - Side effects
- [watch()](./watch.md) - Explicit watchers
- [State Management Guide](../guides/state-management.md)

## References

- [Signals RFC](https://github.com/vela-lang/rfcs/pull/001)
- [Fine-grained Reactivity Paper](https://github.com/solidjs/signals)
```

---

## 7.4 Estándares de Escritura

### 7.4.1 Estilo de Código en Documentación

**Convenciones**:
- Usar bloques de código con syntax highlighting: ` ```vela `
- Incluir comentarios explicativos en ejemplos complejos
- Mostrar output esperado cuando sea relevante
- Usar nombres descriptivos en ejemplos

**Ejemplo de buen código documentado**:
```vela
// Fetch user data from API
async fn fetchUser(userId: Int): Result<User, Error> {
  try {
    // Make HTTP request
    let response = await http.get("/api/users/${userId}");
    
    // Parse JSON response
    let user = await response.json<User>();
    
    // Return successful result
    return Result.Ok(user);
  } catch (e: NetworkError) {
    // Handle network errors
    return Result.Err(Error("Failed to fetch user: ${e.message}"));
  }
}

// Usage example
let result = await fetchUser(123);
match (result) {
  Result.Ok(user) => print("User: ${user.name}"),
  Result.Err(error) => print("Error: ${error}")
}
```

---

### 7.4.2 Tono y Voz

**Principios**:
- **Claro y conciso**: Evitar jerga innecesaria
- **Amigable pero profesional**: No demasiado informal
- **Educativo**: Explicar el "por qué", no solo el "cómo"
- **Inclusivo**: Usar lenguaje neutral

**Ejemplos**:

❌ **Malo**:
> Obviamente, deberías usar signals aquí porque son súper rápidos.

✅ **Bueno**:
> Signals are recommended here because they provide automatic dependency tracking and efficient updates.

---

### 7.4.3 Estructura de Explicaciones

**Patrón recomendado**:
1. **Qué es** (definición corta)
2. **Por qué existe** (problema que resuelve)
3. **Cuándo usarlo** (casos de uso)
4. **Cómo usarlo** (sintaxis y ejemplos)
5. **Consideraciones** (edge cases, performance)

---

## 7.5 Herramientas de Documentación

### 7.5.1 Doc Generator

```bash
# Generar docs desde código fuente
vela doc generate

# Generar docs para API específica
vela doc generate vela.reactive

# Formato de salida
vela doc generate --format=html
vela doc generate --format=markdown
vela doc generate --format=json
```

### 7.5.2 Doc Comments en Código

```vela
/// Crea un signal reactivo con un valor inicial.
///
/// Los signals son la base del sistema reactivo de Vela. Cuando lees
/// el valor de un signal dentro de un `computed()` o `effect()`, 
/// ese signal se trackea automáticamente como dependencia.
///
/// # Ejemplo
/// ```vela
/// let count = signal(0);
/// 
/// effect(() => {
///   print("Count: ${count.value}");
/// });
/// 
/// count.value = 10; // Triggers effect
/// ```
///
/// # Ver también
/// - `computed()` para valores derivados
/// - `effect()` para efectos secundarios
///
/// # Parámetros
/// - `initialValue`: El valor inicial del signal
///
/// # Retorna
/// Un nuevo `Signal<T>`
public fn signal<T>(initialValue: T): Signal<T> {
  // Implementation...
}
```

---

## 7.6 Testing de Documentación

### 7.6.1 Doc Tests

```vela
/// # Ejemplo
/// ```vela
/// let numbers = [1, 2, 3, 4, 5];
/// let doubled = numbers.map((n) => n * 2);
/// assert(doubled == [2, 4, 6, 8, 10]);
/// ```
fn map<T, U>(list: List<T>, fn: (T) => U): List<U> {
  // ...
}
```

Ejecutar doc tests:
```bash
vela test --doc
```

---

## 7.7 Documentación Multilingüe

### 7.7.1 Estructura

```
docs/
├── en/          # English (default)
│   └── ...
├── es/          # Español
│   └── ...
├── fr/          # Français
│   └── ...
└── zh/          # 中文
    └── ...
```

### 7.7.2 Traducción

- Mantener sincronización con versión en inglés
- Usar herramientas de i18n
- Revisar por hablantes nativos

---

## 7.8 Documentación Versionada

### 7.8.1 Estructura por Versión

```
docs.vela-lang.org/
├── latest/      # Latest stable
├── v1.0/
├── v1.1/
├── v2.0/
└── next/        # Development version
```

### 7.8.2 Changelog

```markdown
# Changelog

## [1.1.0] - 2025-06-01

### Added
- New `watch()` API for explicit signal watching
- Support for async signals
- Grid layout widget

### Changed
- Improved performance of signal propagation (30% faster)
- Updated `computed()` to support async functions

### Deprecated
- `oldAPI()` - Use `newAPI()` instead

### Removed
- `legacyFunction()` - Removed as planned

### Fixed
- Bug in actor message ordering (#123)
- Memory leak in signal cleanup (#145)

### Security
- Fixed potential XSS in string interpolation (#167)
```

---

## 7.9 Contributing Guide

```markdown
# Contributing to Vela Documentation

Thank you for your interest in improving Vela's documentation!

## Types of Contributions

1. **Typo fixes** - Small corrections
2. **Clarifications** - Improving existing explanations
3. **New content** - Adding missing documentation
4. **Examples** - Contributing code examples
5. **Translations** - Translating to other languages

## Process

1. Fork the repository
2. Create a branch: `docs/your-topic`
3. Make changes following our style guide
4. Test code examples
5. Submit pull request

## Style Guide

- Follow templates in `/templates/`
- Use clear, concise language
- Include runnable code examples
- Add screenshots for UI features

## Review Process

All documentation PRs are reviewed by:
1. Technical accuracy
2. Writing quality
3. Code examples compilation
4. Accessibility

Usually takes 2-3 days.
```

---

**FIN DEL DOCUMENTO: Documentación Técnica**

Este documento establece el framework completo de documentación para Vela.
