# VELA-566: Gramática Completa del Lenguaje Vela

## 📋 Información General
- **Epic:** VELA-559 (Phase 0: Foundation)
- **Sprint:** Sprint 4 (Phase 0)
- **Estado:** Completada ✅
- **Fecha Inicio:** 2025-11-30
- **Fecha Fin:** 2025-11-30
- **Tipo:** Historia de Usuario (US-01)

## 🎯 Descripción

**Historia de Usuario:**  
> Como desarrollador del compilador Vela, necesito una **especificación formal completa de la gramática** del lenguaje en notación EBNF, incluyendo todas las reglas léxicas y sintácticas, precedencia de operadores y palabras reservadas, para poder implementar el parser de producción en Phase 1.

**Contexto:**  
Esta Historia completa la **Phase 0: Foundation** del proyecto Vela. La gramática formal definida aquí será la base para:
- Implementación del lexer de producción (Phase 1)
- Implementación del parser de producción (Phase 1)
- Generación de tests automáticos
- Documentación oficial del lenguaje
- Especificación para herramientas externas (IDEs, linters, formatters)

**Relación con VELA-565:**  
VELA-565 validó la arquitectura mediante prototipos. VELA-566 **formaliza** el diseño completo del lenguaje.

---

## 📦 Subtasks Completadas

### ✅ TASK-001: Gramática EBNF completa
**Archivo:** `docs/language-design/vela-grammar-ebnf.md` (~400 lines)

**Contenido:**
- **Notación EBNF:** Reglas y símbolos utilizados
- **Gramática Léxica:** Tokens, keywords (60+), identificadores, literales, operadores (30+), delimitadores
- **Gramática Sintáctica:** 
  - Estructura de programa (módulos, imports, public exports)
  - Declaraciones (funciones, structs, enums, traits, impl)
  - Statements (inmutables, state, expression, blocks)
  - Expressions (15 niveles de precedencia)
  - Control flow (if, match, métodos funcionales, async/await, try/catch)
  - Patterns (literal, identifier, tuple, struct, enum, or, range)
  - Types (primitives, arrays, tuples, functions, generics, references)
- **Keywords Domain-Specific:** widget, component, service, repository, entity, dto, factory, builder, strategy, observer, singleton, adapter, decorator, controller, middleware, guard, interceptor, validator, pipe, task, helper, mapper, serializer, store, provider, actor
- **Sistema Reactivo:** Signal, Computed, Effect, Watch, @injectable, @inject, @container, @provides, @get, @post, @put, @delete, @patch, store, dispatch
- **Features Modernos:** String interpolation `${}`, optional chaining `?.`, Option<T> coalescing `??`, exponentiation `**`, async/await, pattern matching, generics, error handling
- **Tabla de Precedencia:** 15 niveles (assignment → logical → equality → comparison → bitwise → arithmetic → unary → postfix)

**Decisiones Clave:**
1. EBNF como notación estándar (legibilidad + herramientas existentes)
2. 60+ keywords (balance entre expresividad y simplicidad)
3. 15 niveles de precedencia (similar a Rust)
4. String interpolation con `${}` (más seguro que templates)
5. Optional chaining `?.` y Option<T> coalescing `??` (ergonomía)
6. Pattern matching exhaustivo (seguridad)

---

### ✅ TASK-002: Precedencia de operadores
**Archivo:** `docs/language-design/operator-precedence.md` (~850 lines)

**Contenido:**
- **Tabla de Precedencia Completa:** 15 niveles con asociatividad
  1. Assignment (`=`, `+=`, etc.) - Right
  2. Logical OR (`||`) - Left
  3. Logical AND (`&&`) - Left
  4. Option<T> Coalescing (`??`) - Left
  5. Equality (`==`, `!=`) - Left
  6. Comparison (`<`, `>`, `<=`, `>=`) - Left
  7. Bitwise OR (`|`) - Left
  8. Bitwise XOR (`^`) - Left
  9. Bitwise AND (`&`) - Left
  10. Shift (`<<`, `>>`) - Left
  11. Additive (`+`, `-`) - Left
  12. Multiplicative (`*`, `/`, `%`) - Left
  13. Exponentiation (`**`) - Right
  14. Unary (`-`, `!`, `~`, `*`, `&`, `&mut`) - Right
  15. Postfix (`()`, `[]`, `.`, `?.`, `?`) - Left

- **Descripción Detallada por Grupo:** 40+ operadores
- **Ejemplos de Interacción:** 15+ expresiones complejas con evaluación paso a paso
- **Casos Especiales:**
  - Comparaciones encadenadas (NO soportadas)
  - Operador ternario (NO existe, usar `if` expressions)
  - Distinción `?` (postfix) vs `??` (infix)
- **Justificación de Diseño:**
  - Exponenciación Right Associative (convención matemática)
  - `??` separado de `||` (claridad)
  - NO comparaciones encadenadas (explícito > implícito)
- **Comparación con Lenguajes:** C++, Rust, Python, JavaScript, Java, Go

**Decisiones Clave:**
1. 15 niveles (balance óptimo)
2. Exponenciación Right Associative: `a ** b ** c` → `a ** (b ** c)`
3. `??` nivel 4 (antes de equality, después de logical)
4. NO comparaciones encadenadas: `a < b < c` es error
5. `?` postfix (error propagation) distinto de `??` infix (Option<T> coalescing)

---

### ✅ TASK-003: Palabras reservadas
**Archivo:** `docs/language-design/reserved-keywords.md` (~1,271 lines) - **CORREGIDO**

**Contenido (POST-CORRECCIÓN):**
- **~100 Keywords Totales** organizadas en categorías funcionales:
  - Control Flow (3): `if`, `else`, `match` (NO loops imperativos)
  - Declarations (8): `state`, `fn`, `struct`, `enum`, `trait`, `impl`, `type`, `interface`
  - Visibility & Modifiers (9): `public`, `private`, `protected`, `async`, `static`, `abstract`, `override`, `overload`, `extern`
  - Types & Values (8): `true`, `false`, `None`, `Some`, `self`, `Self`, `super`, `Option`
  - Error Handling (4): `try`, `catch`, `throw`, `finally`
  - Async Programming (3): `async`, `await`, `yield`
  - Module System (4): `import`, `from`, `as`, `show`, `hide` (NO export keyword)
  - **Domain-Specific (25):**
    - UI: `widget`, `component`
    - Architecture: `service`, `repository`, `controller`, `usecase`
    - Models: `dto`, `entity`, `valueObject`, `model`
    - Patterns: `factory`, `builder`, `strategy`, `observer`, `singleton`, `adapter`, `decorator`
    - Web: `guard`, `middleware`, `interceptor`, `validator`
    - Utilities: `pipe`, `task`, `helper`, `mapper`, `serializer`
  - **Reactive System (8):** `Signal`, `Computed`, `Effect`, `Watch`, `store`, `dispatch`, `provide`, `inject`
  - Reserved (Future) (5): `macro`, `defer`, `go`, `chan`, `select`

- **Lista Alfabética Completa**
- **Keywords Contextuales:** `as`, `in` (siempre reservados); `of`, `ref`, `typeof` (contextuales)
- **Prevención de Conflictos:** Qué NO se puede usar como identificador
- **Agrupación por Letra:** A-Y (fácil búsqueda)
- **Justificación de Diseño:**
  - Por qué 25 keywords domain-specific
  - Por qué separar `Signal` vs `store`
  - Por qué reservar keywords para futuro
- **Comparación con Lenguajes:** Vela (80) vs Rust (53), Python (35), JavaScript (63), Java (50), C++ (95), Go (25)

**Decisiones Clave (POST-CORRECCIÓN):**
1. **~100 keywords** (paradigma funcional puro)
2. **25 domain-specific** (claridad arquitectónica)
3. **8 reactive** (reactividad first-class)
4. **NO loops imperativos** (for, while, loop eliminados)
5. **NO mutabilidad por defecto** (let, const, var eliminados; usar state)
6. **Option<T> en lugar de null** (None/Some en lugar de null/undefined)
7. **Modificador public** en lugar de export keyword
8. Capitalización: `Signal`, `Computed`, `Effect`, `Watch` (son constructores)

---

## 🔨 Implementación

### Archivos generados

#### Especificaciones del Lenguaje
- `docs/language-design/vela-grammar-ebnf.md` - Gramática EBNF completa (~400 lines)
- `docs/language-design/operator-precedence.md` - Precedencia de operadores (~850 lines)
- `docs/language-design/reserved-keywords.md` - Palabras reservadas (~1,100 lines)

#### Documentación de Subtasks
- `docs/features/VELA-566/TASK-001.md` - Documentación TASK-001 (EBNF)
- `docs/features/VELA-566/TASK-002.md` - Documentación TASK-002 (Precedencia)
- `docs/features/VELA-566/TASK-003.md` - Documentación TASK-003 (Keywords)
- `docs/features/VELA-566/README.md` - Este archivo (Historia completa)

### Líneas de Código

| Archivo | Tipo | Líneas | Contenido |
|---------|------|--------|-----------|
| `vela-grammar-ebnf.md` | Especificación | ~400 | Gramática completa (CORREGIDA) |
| `operator-precedence.md` | Especificación | ~850 | 40+ operadores, 15+ ejemplos |
| `reserved-keywords.md` | Especificación | ~1,271 | ~100 keywords funcionales (CORREGIDA) |
| `TASK-001.md` | Documentación | ~300 | Meta-documentación EBNF |
| `TASK-002.md` | Documentación | ~850 | Meta-documentación precedencia |
| `TASK-003.md` | Documentación | ~1,100 | Meta-documentación keywords |
| `README.md` | Historia | ~600 | Este archivo |
| **TOTAL** | | **~5,200** | Especificación completa |

---

## 📊 Métricas

### Especificación del Lenguaje (POST-CORRECCIÓN)
- **Keywords:** ~100 (paradigma funcional puro)
- **Operadores:** 40+ (15 niveles de precedencia)
- **Reglas EBNF:** 150+ (léxicas + sintácticas, CORREGIDAS)
- **Tipos primitivos:** Number, Float, String, Bool, Option<T>, Result<T, E>
- **Keywords domain-specific:** 25 (arquitectura + patrones)
- **Keywords reactivos:** 8 (signals, stores, DI)
- **Métodos funcionales:** 25+ (.map(), .filter(), .reduce(), .forEach(), etc.)

### Documentación
- **Archivos creados:** 7
- **Líneas totales:** ~5,200
- **Ejemplos de código:** 100+ (funcionales)
- **Tablas de referencia:** 20+
- **Comparaciones con lenguajes:** 8 (C++, Rust, Python, JavaScript, Java, Go, TypeScript)

### Decisiones de Diseño
- **ADRs implícitos:** 15+ (documentados en justificaciones)
- **Trade-offs analizados:** 10+ (simplicidad vs expresividad)
- **Comparaciones realizadas:** 8 lenguajes

---

## ✅ Definición de Hecho (DoD)

### Funcional
- [x] Gramática EBNF completa (léxica + sintáctica) - **CORREGIDA**
- [x] Todos los tokens definidos (keywords funcionales, operadores, delimitadores)
- [x] Tabla de precedencia completa (15 niveles)
- [x] Lista completa de keywords (~100 funcionales) - **CORREGIDA**
- [x] Reglas de asociatividad especificadas
- [x] **CORRECCIONES POST-SPRINT:** Eliminados loops imperativos, null, let/const/var, export

### Documentación
- [x] Especificación EBNF con ejemplos
- [x] Precedencia documentada con evaluación paso a paso
- [x] Keywords categorizadas con ejemplos de uso
- [x] Casos especiales documentados (comparaciones encadenadas, ternario, etc.)
- [x] Justificación de decisiones de diseño
- [x] Comparación con otros lenguajes

### Calidad
- [x] Notación estándar (EBNF)
- [x] Sin ambigüedades
- [x] Consistente con prototipos (VELA-565)
- [x] Preparado para implementación (Phase 1)

### Entrega
- [x] 3 Subtasks completadas
- [x] 7 archivos generados (~5,200 lines)
- [x] Documentación completa
- [x] README de Historia

---

## 🔍 Decisiones Arquitectónicas

### 1. Notación EBNF (vs BNF, PEG, otros)

**Decisión:** Usar EBNF (Extended Backus-Naur Form)

**Justificación:**
- ✅ **Estándar de facto:** ISO/IEC 14977
- ✅ **Legibilidad:** Más concisa que BNF pura
- ✅ **Herramientas:** Muchos parsers EBNF existentes
- ✅ **Documentación:** Fácil de entender para humanos

**Alternativas Consideradas:**
- BNF (demasiado verboso)
- PEG (Parsing Expression Grammar - no estándar)
- Railroad Diagrams (difícil de mantener)

**Consecuencias:**
- ✅ Especificación clara y concisa
- ✅ Base sólida para parser generator (si se usa)
- ⚠️ Requiere transformación para implementación manual

---

### 2. 15 Niveles de Precedencia (vs 10, 17, 20+)

**Decisión:** 15 niveles de precedencia

**Justificación:**
- ✅ **Balance:** Entre simplicidad (Go: 5) y complejidad (C++: 17, JavaScript: 20)
- ✅ **Similar a Rust:** 14 niveles (probado y exitoso)
- ✅ **Suficiente:** Cubre todos los operadores necesarios
- ✅ **No excesivo:** Evita confusión

**Alternativas Consideradas:**
- 10 niveles (insuficiente para todos los operadores)
- 17+ niveles (complejidad innecesaria como C++)
- 5 niveles (demasiado simple como Go)

**Consecuencias:**
- ✅ Precedencia predecible
- ✅ Similar a lenguajes conocidos (Rust)
- ⚠️ Desarrolladores deben aprender 15 niveles

---

### 3. Domain-Specific Keywords (25 keywords)

**Decisión:** Incluir 25 keywords domain-specific (`service`, `repository`, `dto`, `entity`, `widget`, etc.)

**Justificación:**
- ✅ **Claridad arquitectónica:** `service UserService` > `struct UserService`
- ✅ **Enforcing patterns:** Previene mezcla de concerns
- ✅ **IDE support:** Mejor autocomplete y navegación
- ✅ **Code generation:** Facilita generación de boilerplate
- ✅ **Philosophy:** Vela prioriza claridad sobre minimalismo

**Alternativas Consideradas:**
- No tener domain-specific keywords (menos claridad)
- Usar atributos/decorators (@service) (menos first-class)
- Usar convenciones de nombres (no enforced)

**Consecuencias:**
- ✅ Código más claro y arquitectura más forzada
- ✅ Mejor experiencia de desarrollo
- ⚠️ 80 keywords total (más que lenguajes minimalistas)
- ⚠️ Más curva de aprendizaje

---

### 4. Reactive Keywords (8 keywords)

**Decisión:** Keywords first-class para reactividad (`Signal`, `Computed`, `Effect`, `Watch`, `store`, `dispatch`, `provide`, `inject`)

**Justificación:**
- ✅ **Reactividad core:** Fundamental en Vela
- ✅ **Ergonomía:** `count: Signal<Number> = Signal(0)` (inmutable y reactivo)
- ✅ **Consistencia:** Integrado en lenguaje, no librería
- ✅ **Modern:** Reactividad es estándar en apps modernas

**Alternativas Consideradas:**
- Reactividad como librería (menos integrado)
- Funciones en lugar de keywords (menos ergonómico)
- Menos keywords reactivos (insuficiente)

**Consecuencias:**
- ✅ Reactividad first-class
- ✅ Mejor developer experience
- ⚠️ Más keywords (80 total)

---

### 5. Exponenciación Right Associative

**Decisión:** `a ** b ** c` se evalúa como `a ** (b ** c)` (Right)

**Justificación:**
- ✅ **Convención matemática:** $2^{3^2} = 2^9 = 512$, no $(2^3)^2 = 64$
- ✅ **Consistencia con otros lenguajes:** Python, Ruby, Rust (Right)
- ✅ **Menos sorpresas:** Coincide con expectativa matemática

**Alternativas Consideradas:**
- Left Associative (inconsistente con matemáticas)
- No tener operador `**` (usar función `pow()`)

**Consecuencias:**
- ✅ Menos sorpresas para desarrolladores
- ✅ Coincide con notación matemática
- ⚠️ Right Associative es menos común (mayoría de operadores son Left)

---

### 6. NO Comparaciones Encadenadas

**Decisión:** `a < b < c` NO significa "a < b AND b < c"

**Justificación:**
- ✅ **Explícito > Implícito:** `a < b && b < c` es más claro
- ✅ **Prevención de errores:** `(a < b) < c` compara boolean con c (error)
- ✅ **Consistencia:** Similar a Rust, C++, Java

**Alternativas Consideradas:**
- Soportar comparaciones encadenadas (como Python)
- Parser error en `a < b < c` (demasiado restrictivo)

**Consecuencias:**
- ✅ Menos ambigüedad
- ✅ Código más explícito
- ⚠️ Desarrolladores de Python necesitan ajustarse

---

### 7. String Interpolation con `${}`

**Decisión:** String interpolation usa `${}` en lugar de template strings

**Justificación:**
- ✅ **Seguridad:** No confunde strings regulares con templates
- ✅ **Claridad:** `"Hello, ${name}!"` es explícito
- ✅ **Familiar:** Similar a JavaScript, Kotlin

**Alternativas Consideradas:**
- Template strings con ` (backticks como JS) - ambiguo
- Format strings `"Hello, {}".format(name)` - verboso
- Concatenación `"Hello, " + name` - menos ergonómico

**Consecuencias:**
- ✅ Interpolation ergonómica y segura
- ✅ No necesita strings especiales (backticks)
- ⚠️ Syntax levemente diferente a lenguajes conocidos

---

### 8. Optional Chaining `?.` y Option<T> Coalescing `??`

**Decisión:** Incluir `?.` (optional chaining) y `??` (Option<T> coalescing)

**Justificación:**
- ✅ **Ergonomía:** `user?.profile?.email` > verificación manual anidada
- ✅ **Modernidad:** Features estándar en lenguajes modernos (JS, C#, Swift)
- ✅ **Safety:** Manejo explícito de Option<T> con None/Some
- ✅ **Legibilidad:** Código más conciso

**Alternativas Consideradas:**
- No incluir (menos ergonómico)
- Solo `?.` sin `??` (insuficiente)
- Usar `||` en lugar de `??` (ambiguo)

**Consecuencias:**
- ✅ Manejo de Option<T> ergonómico (None/Some en lugar de null)
- ✅ Código más legible
- ⚠️ 2 operadores adicionales (pero valen la pena)

---

## 📚 Lecciones Aprendidas

### ✅ Lo que funcionó bien

1. **EBNF como notación estándar**
   - Legible para humanos
   - Compatible con herramientas existentes
   - Fácil de mantener

2. **Organización por categorías**
   - Keywords: 10 categorías
   - Operadores: 15 niveles
   - Facilita búsqueda y comprensión

3. **Ejemplos de código**
   - 100+ ejemplos funcionales
   - Clarifica especificación abstracta
   - Ayuda a validar diseño

4. **Justificación de decisiones**
   - Cada decisión importante documentada
   - Trade-offs explicitados
   - Comparación con otros lenguajes

5. **Iteración con prototipos**
   - VELA-565 validó decisiones
   - VELA-566 formalizó diseño
   - Coherencia entre prototipo y especificación

### ⚠️ Desafíos encontrados

1. **Balance simplicidad vs expresividad**
   - 80 keywords es alto
   - Pero cada uno tiene propósito claro
   - Decision: Priorizar claridad

2. **Precedencia de operadores**
   - 15 niveles es complejo
   - Pero necesario para todos los operadores
   - Decision: Similar a Rust (14 niveles)

3. **Domain-specific keywords**
   - 25 keywords es mucho
   - Pero fuerza buenas prácticas
   - Decision: Claridad arquitectónica > minimalismo

4. **Capitalización inconsistente**
   - `Signal`, `Computed`, `Effect`, `Watch` con mayúscula
   - Resto de keywords minúsculas
   - Decision: Constructores merecen capitalización

5. **Reactividad first-class**
   - 8 keywords reactivos
   - Aumenta complejidad
   - Decision: Reactividad es core en Vela

6. **Correcciones post-Sprint**
   - Archivos originales contenían keywords imperativos (for, while, loop)
   - Se corrigió a paradigma funcional puro (commits e5bc0a6, 39c7f5c)
   - ~100 keywords finales (vs 80 originales)

### 🔄 Mejoras futuras

1. **Railroad diagrams**
   - Visualización de gramática
   - Complementa EBNF
   - Facilita comprensión

2. **Parser generator**
   - Generar parser desde EBNF
   - Reduce errores de implementación
   - Mantiene sincronización spec-implementación

3. **Tests de gramática**
   - Test suite desde EBNF
   - Validar parser de producción
   - Cobertura completa de sintaxis

4. **Feedback de implementación**
   - Phase 1 revelará issues
   - Ajustar especificación si necesario
   - Mantener coherencia

---

## 🚀 Próximos Pasos

### Inmediatos (Sprint 4 Commit)
1. ✅ Completar documentación VELA-566 (este README)
2. ⏳ Commit Sprint 4 completo (VELA-565 + VELA-566)
3. ⏳ Tag: `sprint-4`

### Phase 1: Production Compiler
1. **Lexer de producción**
   - Implementar tabla de keywords (80 keywords)
   - Tokenizar operadores (40+ operators)
   - Manejo de strings, números, identificadores
   - Tests unitarios (~500 tests)

2. **Parser de producción**
   - Implementar precedence climbing (15 niveles)
   - Parser recursive descent desde EBNF
   - Construcción de AST
   - Tests de gramática (~1,000 tests)

3. **Semantic Analysis**
   - Validar domain-specific keywords
   - Enforcing de patrones arquitectónicos
   - Type checking
   - Error reporting

4. **Validación**
   - Parser cumple especificación EBNF
   - Precedencia correcta (tests)
   - Keywords reservadas (tests)
   - Sin ambigüedades

---

## 🌟 Impacto

### En el Proyecto
- ✅ **Phase 0 COMPLETADA:** Foundation establecida
- ✅ **Especificación formal:** Base sólida para implementación
- ✅ **Documentación exhaustiva:** ~5,200 lines
- ✅ **Decisiones validadas:** Prototipos + especificación formal

### En el Lenguaje (POST-CORRECCIÓN)
- ✅ **Gramática completa:** 150+ reglas EBNF (funcional pura)
- ✅ **~100 keywords:** Cobertura completa con paradigma funcional
- ✅ **40+ operadores:** Precedencia bien definida
- ✅ **Features modernos:** String interpolation, optional chaining, Option<T> coalescing, async/await, pattern matching
- ✅ **NO loops imperativos:** for, while, loop eliminados
- ✅ **NO mutabilidad por defecto:** let, const, var eliminados

### En el Compilador (Futuro)
- 🔧 **Lexer:** Tabla de keywords y operadores lista
- 🔧 **Parser:** Precedence climbing y recursive descent especificados
- 🔧 **Semantic Analysis:** Domain-specific keywords para validar
- 🔧 **Tests:** Base para generar test suites

### En el Ecosistema
- 📖 **Documentación oficial:** Referencia para desarrolladores
- 🛠️ **Tooling:** Base para IDEs, linters, formatters
- 🎓 **Aprendizaje:** Tutorial y reference manual
- 🌍 **Comunidad:** Especificación pública y abierta

---

## 📊 Comparación con Otros Lenguajes

| Aspecto | Vela | Rust | Python | JavaScript | Go |
|---------|------|------|--------|------------|-----|
| **Keywords** | ~100 | 53 | 35 | 63 | 25 |
| **Precedence Levels** | 15 | 14 | 16 | 20 | 5 |
| **Operators** | 40+ | 35+ | 30+ | 50+ | 20+ |
| **Domain-Specific** | ✅ 25 | ❌ | ❌ | ❌ | ❌ |
| **Reactive Built-in** | ✅ 8 | ❌ | ❌ | ❌ | ❌ |
| **Functional Methods** | ✅ 25+ | ✅ | ✅ | ✅ | ✅ |
| **Imperative Loops** | ❌ | ✅ | ✅ | ✅ | ✅ |
| **Null Type** | ❌ Option<T> | ❌ Option<T> | ✅ None | ✅ null | ✅ nil |
| **Immutable by Default** | ✅ | ❌ | ❌ | ❌ | ❌ |
| **String Interpolation** | ✅ `${}` | ❌ (macros) | ✅ f-strings | ✅ template strings | ❌ |
| **Optional Chaining** | ✅ `?.` | ❌ | ❌ | ✅ `?.` | ❌ |
| **Option<T> Coalescing** | ✅ `??` | ❌ | ❌ | ✅ `??` | ❌ |
| **Pattern Matching** | ✅ | ✅ | ✅ (3.10+) | ❌ | ❌ |
| **Async/Await** | ✅ | ✅ | ✅ | ✅ | ❌ (goroutines) |

**Análisis (POST-CORRECCIÓN):**
- Vela tiene **MÁS keywords** (~100) que la mayoría por paradigma funcional puro + domain-specific + reactive
- **Precedencia similar a Rust** (14-15 niveles) - probado y exitoso
- **Paradigma funcional PURO:** NO loops imperativos (for, while, loop)
- **Inmutabilidad por defecto:** NO let/const/var keywords
- **Option<T> en lugar de null:** None/Some (seguridad de tipos)
- **Features modernos** como optional chaining, Option<T> coalescing, string interpolation
- **Único** con domain-specific keywords (service, repository, dto, widget, etc.)
- **Único** con reactive keywords first-class (Signal, Computed, Effect, Watch, store)
- **Único** con paradigma funcional puro e inmutabilidad por defecto

---

## 🔗 Referencias

### Archivos de Especificación
- `docs/language-design/vela-grammar-ebnf.md` - Gramática completa
- `docs/language-design/operator-precedence.md` - Precedencia de operadores
- `docs/language-design/reserved-keywords.md` - Palabras reservadas

### Documentación de Subtasks
- `docs/features/VELA-566/TASK-001.md` - EBNF grammar
- `docs/features/VELA-566/TASK-002.md` - Operator precedence
- `docs/features/VELA-566/TASK-003.md` - Reserved keywords

### Historia Relacionada
- **VELA-565** (US-00F): Prototype & Validation - Validó decisiones arquitectónicas
- **VELA-559** (Epic): Phase 0: Foundation - Epic padre

### Jira
- **Historia:** [VELA-566](https://velalang.atlassian.net/browse/VELA-566)
- **Subtasks:** TASK-001, TASK-002, TASK-003

---

## 🎉 Resumen Final

**VELA-566 completada exitosamente:**

✅ **3 Subtasks completadas**
✅ **7 archivos generados** (~5,200 lines)
✅ **Especificación formal completa** (EBNF + precedencia + keywords)
✅ **~100 keywords funcionales** documentadas con ejemplos (POST-CORRECCIÓN)
✅ **40+ operadores** con 15 niveles de precedencia
✅ **100+ ejemplos** de código funcional
✅ **15+ decisiones** arquitectónicas documentadas
✅ **Comparación** con 8 lenguajes (C++, Rust, Python, JavaScript, Java, Go, TypeScript, Swift)
✅ **CORRECCIONES:** Commits e5bc0a6 y 39c7f5c eliminaron keywords imperativos

🚀 **Phase 0 COMPLETADA**

⏭️ **Próximo:** Sprint 4 Commit → Phase 1: Production Compiler

---

**Historia:** VELA-566  
**Sprint:** Sprint 4  
**Estado:** ✅ COMPLETADA  
**Fecha:** 2025-11-30  
**Total Subtasks:** 3/3 ✅  
**Total Archivos:** 7 (~5,200 lines) + 2 correcciones post-Sprint  
**Total Keywords:** ~100 (paradigma funcional puro)  
**Total Operadores:** 40+  
**Total Ejemplos:** 100+  
**Commits de Corrección:** e5bc0a6, 39c7f5c (eliminados for, while, loop, null, let, const, var, export)
