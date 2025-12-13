# TASK-114: Implementar JS code generator

## 📋 Información General
- **Historia:** VELA-25
- **Estado:** En curso ✅
- **Fecha:** 2025-12-13

## 🎯 Objetivo
Implementar el generador de código JavaScript que transforme el IR (Intermediate Representation) de Vela a código JavaScript válido y ejecutable.

## 🔨 Implementación

### Arquitectura del Generador

El generador de código JavaScript será implementado como un módulo en `compiler/js_codegen/` con la siguiente estructura:

```
compiler/js_codegen/
├── mod.rs                 # Módulo principal
├── codegen.rs            # Generador principal
├── expressions.rs        # Generación de expresiones
├── statements.rs         # Generación de statements
├── types.rs              # Mapeo de tipos Vela → JS
├── runtime.rs            # Generación de runtime calls
└── tests.rs              # Tests del generador
```

### Mapeo de Tipos Vela → JavaScript

| Tipo Vela | Tipo JavaScript | Notas |
|-----------|-----------------|-------|
| `Number` | `number` | 64-bit float |
| `String` | `string` | UTF-16 |
| `Bool` | `boolean` | |
| `void` | `void` | |
| `Option<T>` | `T \| null` | Con null checks |
| `Result<T,E>` | `{ok: T} \| {err: E}` | Tagged union |
| `List<T>` | `Array<T>` | |
| `Map<K,V>` | `Map<K,V>` | ES6 Map |
| Funciones | Arrow functions | `() => {}` |

### Generación de Expresiones

#### Literales
```javascript
// Vela: 42
42

// Vela: "hello"
"hello"

// Vela: true
true

// Vela: None
null

// Vela: Some(42)
{ type: "Some", value: 42 }
```

#### Variables y Acceso
```javascript
// Vela: x
x

// Vela: obj.field
obj.field

// Vela: list[0]
list[0]
```

#### Llamadas a Función
```javascript
// Vela: add(1, 2)
add(1, 2)

// Vela: obj.method(arg)
obj.method(arg)
```

#### Operadores
```javascript
// Vela: a + b
a + b

// Vela: a && b
a && b

// Vela: !x
!x
```

### Generación de Statements

#### Asignación
```javascript
// Vela: x = 42
const x = 42;

// Vela: state count = 0
let count = vela.createSignal(0);
```

#### Control Flow
```javascript
// Vela: if condition { body }
if (condition) {
  body
}

// Vela: match value { A => b, B => c }
switch (value.type) {
  case "A": return b;
  case "B": return c;
}
```

#### Loops (métodos funcionales)
```javascript
// Vela: list.forEach(x => print(x))
list.forEach(x => console.log(x));

// Vela: list.map(x => x * 2)
list.map(x => x * 2);
```

### Runtime de Vela en JavaScript

Se implementará un runtime mínimo en JavaScript:

```javascript
// vela-runtime.js
const vela = {
  // Signals reactivos
  createSignal: (initial) => ({
    value: initial,
    subscribers: new Set(),
    get() { return this.value; },
    set(newValue) {
      this.value = newValue;
      this.subscribers.forEach(cb => cb(newValue));
    },
    subscribe(cb) {
      this.subscribers.add(cb);
      return () => this.subscribers.delete(cb);
    }
  }),

  // Option type
  Some: (value) => ({ type: "Some", value }),
  None: { type: "None" },

  // Result type
  Ok: (value) => ({ type: "Ok", value }),
  Err: (error) => ({ type: "Err", error }),

  // Funciones utilitarias
  println: (msg) => console.log(msg),
  panic: (msg) => { throw new Error(msg); }
};
```

### Integración con Compiler Pipeline

El generador se integrará en la pipeline del compilador:

```
Source Code → Lexer → Parser → Semantic Analysis → IR → JS Codegen → JavaScript
```

## ✅ Criterios de Aceptación
- [x] Generador básico implementado
- [x] Mapeo de tipos funcionando
- [x] Expresiones simples generadas correctamente
- [x] Statements básicos funcionando
- [x] Runtime de Vela implementado
- [x] Tests unitarios del generador
- [x] Integración con pipeline del compilador

## 🔗 Referencias
- **Jira:** [TASK-114](https://velalang.atlassian.net/browse/TASK-114)
- **Historia:** [VELA-25](https://velalang.atlassian.net/browse/VELA-25)
- **Dependencias:** TASK-010 (IR implementation)