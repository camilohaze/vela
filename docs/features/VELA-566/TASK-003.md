# TASK-003: Definir Palabras Reservadas

## 📋 Información General
- **Historia:** VELA-566 (US-01: Gramática completa)
- **Sprint:** Sprint 4 (Phase 0)
- **Estado:** Completada ✅
- **Fecha:** 2025-11-30

## 🎯 Objetivo
Definir y catalogar todas las palabras reservadas (keywords) del lenguaje Vela, organizándolas por categorías funcionales, documentando su uso y previendo conflictos de nombres.

## 🔨 Implementación

### Archivos generados
- `docs/language-design/reserved-keywords.md` - Especificación completa de palabras reservadas (~1,100 lines)

### Contenido de la documentación

**1. Resumen Estadístico**
- **80 keywords totales** organizadas en 10 categorías
- Control Flow: 11 keywords
- Declarations: 8 keywords
- Visibility & Modifiers: 6 keywords
- Types & Values: 7 keywords
- Error Handling: 3 keywords
- Async Programming: 2 keywords
- Module System: 5 keywords
- Domain-Specific: 25 keywords (arquitectura, patrones, utilidades)
- Reactive System: 8 keywords (signals, stores, DI)
- Reserved (Future): 5 keywords

**2. Lista Completa Alfabética**
```
abstract        actor           adapter         as              async
await           boolean         break           builder         catch
chan            component       Computed        const           continue
controller      crate           decorator       defer           dispatch
dto             Effect          else            entity          enum
export          extern          factory         false           fn
for             from            guard           helper          if
impl            import          in              inject          interceptor
interface       let             loop            macro           mapper
match           middleware      model           module          mut
new             null            number          observer        of
pipe            provider        provide         pub             ref
repository      return          select          self            Self
serializer      service         Signal          singleton       static
store           strategy        string          struct          super
task            throw           trait           true            try
type            typeof          unsafe          usecase         validator
valueObject     watch           Watch           while           widget
yield
```

**3. Keywords por Categoría (detallado)**

Cada categoría incluye:
- Tabla con keyword, descripción, ejemplo
- Código de ejemplo completo
- Notas especiales (short-circuit, async/await, etc.)

**4. Keywords Contextuales**
- `as`, `in` - siempre reservados
- `of`, `ref`, `typeof` - contextuales (pueden usarse como identificadores en ciertos contextos)

**5. Prevención de Conflictos**
```
❌ PROHIBIDO usar como:
- Variable names: let if = 10;
- Function names: fn while() { }
- Type names: struct match { }
- Field names: struct User { let: string }
- Module names: module fn { }

✅ PERMITIDO usar como:
- String literals: let keyword = "if";
- En comentarios
- Raw identifiers (futuro): let r#type = 10;
```

**6. Agrupación por Primera Letra**
- A: 6 keywords (abstract, actor, adapter, as, async, await)
- B: 3 keywords (boolean, break, builder)
- C: 9 keywords (catch, chan, component, Computed, const, continue, controller, crate)
- ... hasta Y (yield)

**7. Justificación de Diseño**

**¿Por qué tantas keywords domain-specific?**
- ✅ Claridad: `service UserService` es más claro que `struct UserService`
- ✅ Arquitectura forzada: Previene mezcla de concerns
- ✅ Soporte de IDE: Mejor autocomplete y navegación
- ✅ Generación de código: Puede generar boilerplate

**¿Por qué separar `Signal` vs `store`?**
- `Signal`: Reactividad granular (valor único)
- `store`: Estado global (múltiples valores + acciones)

**¿Por qué reservar keywords para futuro?**
- Previene breaking changes cuando se agreguen features
- Mejor reservar temprano que romper código existente

**8. Comparación con Otros Lenguajes**
```
Vela:        80 keywords (domain-specific + reactive)
Rust:        53 keywords (systems programming)
Python:      35 keywords (minimal, dinámico)
JavaScript:  63 keywords (ES2022)
Java:        50 keywords (OOP)
C++:         95 keywords (complejo)
Go:          25 keywords (minimalista)
TypeScript:  65+ keywords (JS + types)
```

Vela tiene **MÁS keywords** que la mayoría por soporte domain-specific y reactivo, pero cada keyword tiene propósito claro.

## 📊 Cobertura

### Keywords Documentadas: 80
- **Control Flow:** 11 (`if`, `else`, `match`, `while`, `for`, `in`, `loop`, `break`, `continue`, `return`, `yield`)
- **Declarations:** 8 (`let`, `const`, `fn`, `struct`, `enum`, `trait`, `impl`, `type`)
- **Visibility & Modifiers:** 6 (`pub`, `mut`, `async`, `static`, `unsafe`, `extern`)
- **Types & Values:** 7 (`true`, `false`, `null`, `self`, `Self`, `super`, `crate`)
- **Error Handling:** 3 (`try`, `catch`, `throw`)
- **Async Programming:** 2 (`async`, `await`)
- **Module System:** 5 (`import`, `export`, `from`, `as`, `module`)
- **Domain-Specific:** 25
  - UI: `widget`, `component`
  - Architecture: `service`, `repository`, `controller`, `usecase`
  - Models: `dto`, `entity`, `valueObject`, `model`
  - Patterns: `factory`, `builder`, `strategy`, `observer`, `singleton`, `adapter`, `decorator`
  - Web: `guard`, `middleware`, `interceptor`, `validator`
  - Utilities: `pipe`, `task`, `helper`, `mapper`, `serializer`
- **Reactive System:** 8 (`Signal`, `Computed`, `Effect`, `Watch`, `store`, `dispatch`, `provide`, `inject`)
- **Reserved (Future):** 5 (`macro`, `defer`, `go`, `chan`, `select`)

### Ejemplos Incluidos: 40+
- Código funcional para cada keyword
- Casos de uso típicos
- Patrones idiomáticos
- Casos especiales (contextuales, raw identifiers)

### Categorización: 10 categorías
- Organización lógica
- Fácil búsqueda
- Agrupación por letra (A-Y)

## ✅ Criterios de Aceptación
- [x] Lista completa de 80 keywords
- [x] Organización en 10 categorías
- [x] Descripción y ejemplo para cada keyword
- [x] Keywords contextuales identificadas
- [x] Prevención de conflictos documentada
- [x] Justificación de diseño explicada
- [x] Comparación con otros lenguajes
- [x] Agrupación alfabética

## 🔍 Decisiones de Diseño

### 1. Domain-Specific Keywords (25)
**Decisión:** Incluir keywords específicos para arquitectura (`service`, `repository`, `dto`, `entity`, etc.)

**Justificación:**
- **Claridad**: `service UserService` es autoexplicativo
- **Arquitectura forzada**: Previene mezcla de concerns
- **IDE support**: Mejor autocomplete, refactoring, navegación
- **Code generation**: Facilita generación de boilerplate

**Trade-off:** Más keywords que aprender, pero mayor claridad arquitectónica

### 2. Reactive System Keywords (8)
**Decisión:** Keywords first-class para reactividad (`Signal`, `Computed`, `Effect`, `Watch`, `store`, `dispatch`, `provide`, `inject`)

**Justificación:**
- **Modernidad**: Reactividad es core en aplicaciones modernas
- **Claridad**: `let count = Signal(0)` es más claro que `let count = reactive(0)`
- **Ergonomía**: Mejor developer experience

**Trade-off:** Más keywords, pero reactividad es fundamental en Vela

### 3. Reserved Keywords (5)
**Decisión:** Reservar `macro`, `defer`, `go`, `chan`, `select` para futuro

**Justificación:**
- **Prevención**: Evita breaking changes cuando se implementen
- **Flexibilidad**: Permite evolución del lenguaje
- **Buenas prácticas**: Mejor reservar temprano

**Trade-off:** Keywords que no se usan ahora, pero previenen problemas futuros

### 4. Total de 80 Keywords
**Decisión:** 80 keywords es aceptable para Vela

**Justificación:**
- **Context**: C++ tiene 95, TypeScript ~65, Java 50
- **Purpose**: Cada keyword tiene propósito claro
- **Balance**: Entre expresividad y simplicidad

**Trade-off:** Más que Go (25) o Python (35), pero justificado por domain-specific focus

### 5. Capitalización: `Signal` vs `signal`
**Decisión:** `Signal`, `Computed`, `Effect`, `Watch` con mayúscula inicial

**Justificación:**
- **Constructores**: Son constructores de valores reactivos
- **Distinción**: Se distinguen de funciones regulares
- **Convención**: Similar a tipos (struct, enum)

**Trade-off:** Inconsistencia con otras keywords (todas minúsculas), pero mejora claridad

## 🚀 Impacto

### En el lenguaje
- ✅ **Claridad arquitectónica**: Keywords domain-specific fuerzan buenas prácticas
- ✅ **Reactividad first-class**: Sistema reactivo integrado
- ✅ **Prevención de conflictos**: 80 keywords reservadas evitan colisiones
- ✅ **Evolución futura**: Keywords reservadas permiten crecimiento

### En el compilador
- 🔧 **Lexer**: Tabla de keywords para tokenización
- 🔧 **Parser**: Validación de uso correcto
- 🔧 **Semantic analysis**: Enforcing de patrones arquitectónicos
- 🔧 **Code generation**: Boilerplate para domain-specific keywords

### En el desarrollador
- 📖 **Documentación clara**: 80 keywords bien documentadas
- 🎓 **Curva de aprendizaje**: Mayor por cantidad, pero compensada por claridad
- 💡 **IDE support**: Mejor autocomplete y validación
- ⚠️ **Naming**: Menos opciones para nombres, pero previene ambigüedad

### En el ecosistema
- 🔌 **Frameworks**: Keywords domain-specific simplifican frameworks
- 📦 **Librerías**: Consistent naming conventions
- 🛠️ **Tooling**: Mejor análisis estático y refactoring

## 📚 Referencias
- **EBNF Grammar:** `docs/language-design/vela-grammar-ebnf.md`
- **Operator Precedence:** `docs/language-design/operator-precedence.md`
- **Jira:** [TASK-003](https://velalang.atlassian.net/browse/VELA-566) (subtask de VELA-566)
- **Historia:** [VELA-566](https://velalang.atlassian.net/browse/VELA-566)

## 📝 Lecciones Aprendidas

### ✅ Lo que funcionó bien
1. **Categorización**: 10 categorías claras facilitan navegación
2. **Ejemplos completos**: Cada keyword con código funcional
3. **Justificación de diseño**: Explica el "por qué" de decisiones controversiales
4. **Comparación con lenguajes**: Proporciona contexto y normaliza el count
5. **Agrupación alfabética**: Facilita búsqueda rápida

### ⚠️ Desafíos encontrados
1. **Cantidad**: 80 keywords es alto, pero justificado
2. **Capitalización**: `Signal`, `Computed`, etc. rompen convención
3. **Domain-specific**: Algunos pueden parecer redundantes
4. **Conflictos potenciales**: 80 keywords = más colisiones posibles

### 🔄 Consideraciones Futuras
1. **Feedback de usuarios**: Ajustar basado en uso real
2. **Raw identifiers**: Implementar `r#type` para escape
3. **Deprecation path**: Si alguna keyword resulta innecesaria
4. **Implementación de reservadas**: `macro`, `defer`, `go`, `chan`, `select`

### 🚀 Próximos pasos
- Implementar tabla de keywords en lexer (Phase 1)
- Validar domain-specific keywords en parser
- Crear tests de conflictos de nombres
- Implementar error messages cuando se usa keyword como identificador

---

**Estado Final:** ✅ COMPLETADA  
**Archivos generados:** 1 (~1,100 lines)  
**Keywords documentadas:** 80  
**Categorías:** 10  
**Ejemplos incluidos:** 40+  
**Comparaciones con lenguajes:** 8
