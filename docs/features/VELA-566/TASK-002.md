# TASK-002: Documentar Precedencia de Operadores

## 📋 Información General
- **Historia:** VELA-566 (US-01: Gramática completa)
- **Sprint:** Sprint 4 (Phase 0)
- **Estado:** Completada ✅
- **Fecha:** 2025-11-30

## 🎯 Objetivo
Documentar de manera exhaustiva la precedencia y asociatividad de todos los operadores del lenguaje Vela, incluyendo ejemplos de evaluación, casos especiales y comparación con otros lenguajes.

## 🔨 Implementación

### Archivos generados
- `docs/language-design/operator-precedence.md` - Especificación completa de precedencia (~850 lines)

### Contenido de la documentación

**1. Tabla de Precedencia (15 niveles)**
- Nivel 1: Asignación (`=`, `+=`, etc.) - Right
- Nivel 2: OR lógico (`||`) - Left
- Nivel 3: AND lógico (`&&`) - Left
- Nivel 4: Null coalescing (`??`) - Left
- Nivel 5: Igualdad (`==`, `!=`) - Left
- Nivel 6: Comparación (`<`, `>`, `<=`, `>=`) - Left
- Nivel 7: OR bit a bit (`|`) - Left
- Nivel 8: XOR bit a bit (`^`) - Left
- Nivel 9: AND bit a bit (`&`) - Left
- Nivel 10: Desplazamientos (`<<`, `>>`) - Left
- Nivel 11: Aditivos (`+`, `-`) - Left
- Nivel 12: Multiplicativos (`*`, `/`, `%`) - Left
- Nivel 13: Exponenciación (`**`) - Right
- Nivel 14: Unarios (`-`, `!`, `~`, `*`, `&`) - Right
- Nivel 15: Postfijos (`()`, `[]`, `.`, `?.`, `?`) - Left

**2. Descripción detallada de cada grupo**
- Operadores incluidos
- Asociatividad (Left/Right)
- Ejemplos de evaluación
- Casos de uso típicos
- Notas especiales (short-circuit, etc.)

**3. Ejemplos de interacción**
- Expresiones aritméticas mixtas
- Operadores lógicos combinados
- Cadenas de asignación
- Safe navigation con null coalescing
- Expresiones complejas paso a paso

**4. Casos especiales documentados**
- Comparaciones encadenadas (NO soportadas)
- Operador ternario (NO existe, usar `if` expressions)
- Distinción entre `?` (postfix) y `??` (infix)

**5. Justificación de diseño**
- Por qué exponenciación es Right Associative
- Por qué `??` está separado de `||`
- Por qué NO hay comparaciones encadenadas
- Comparación con C++, Rust, Python, JavaScript, Java

**6. Tabla comparativa**
```
Vela:       15 niveles (limpio, predecible)
C/C++:      17 niveles (complejo, propenso a errores)
Rust:       14 niveles (similar a Vela)
Python:     16 niveles (tiene comparaciones encadenadas)
JavaScript: 20 niveles (muy complejo)
Java:       16 niveles (similar a C)
```

## 📊 Cobertura

### Operadores documentados: 40+
- **Asignación:** 12 operadores (`=`, `+=`, `-=`, `*=`, `/=`, `%=`, `**=`, `&=`, `|=`, `^=`, `<<=`, `>>=`)
- **Lógicos:** 2 operadores (`||`, `&&`)
- **Null handling:** 1 operador (`??`)
- **Comparación:** 6 operadores (`==`, `!=`, `<`, `>`, `<=`, `>=`)
- **Bit a bit:** 5 operadores (`|`, `^`, `&`, `<<`, `>>`)
- **Aritméticos:** 6 operadores (`+`, `-`, `*`, `/`, `%`, `**`)
- **Unarios:** 6 operadores (`-`, `!`, `~`, `*`, `&`, `&mut`)
- **Postfijos:** 5 operadores (`()`, `[]`, `.`, `?.`, `?`)

### Ejemplos incluidos: 15+
- Evaluación paso a paso
- Casos de short-circuit
- Uso de paréntesis para claridad
- Patrones idiomáticos

### Casos especiales documentados: 3
- Comparaciones encadenadas
- Operador ternario (alternativa)
- Distinción `?` vs `??`

## ✅ Criterios de Aceptación
- [x] Tabla de precedencia completa (15 niveles)
- [x] Asociatividad especificada para cada nivel
- [x] Descripción detallada de cada grupo de operadores
- [x] Ejemplos de evaluación paso a paso
- [x] Casos especiales documentados
- [x] Justificación de decisiones de diseño
- [x] Comparación con otros lenguajes

## 🔍 Decisiones de Diseño

### 1. Exponenciación Right Associative
**Decisión:** `a ** b ** c` se evalúa como `a ** (b ** c)`

**Justificación:** Coincide con la convención matemática: $2^{3^2} = 2^9 = 512$

### 2. Null Coalescing separado de OR lógico
**Decisión:** `??` es un operador distinto de `||` con precedencia diferente

**Justificación:**
- `||` es para lógica booleana con short-circuit
- `??` es específicamente para manejo de null/undefined
- Niveles de precedencia separados evitan confusión

### 3. NO hay comparaciones encadenadas
**Decisión:** `a < b < c` NO significa "a < b AND b < c"

**Justificación:**
- Explícito es mejor que implícito
- `a < b && b < c` es más claro
- Evita confusión con booleanos

### 4. 15 niveles de precedencia (no más)
**Decisión:** Mantener 15 niveles en lugar de 17+ como C/C++

**Justificación:**
- Balance entre flexibilidad y simplicidad
- Similar a Rust (14 niveles)
- Más limpio que JavaScript (20 niveles)

## 🚀 Impacto

### En el lenguaje
- ✅ Precedencia clara y predecible
- ✅ Menos paréntesis necesarios
- ✅ Compatibilidad con intuición matemática
- ✅ Prevención de errores comunes

### En el compilador
- 🔧 Parser puede implementar precedence climbing
- 🔧 Tabla de precedencia directa para implementación
- 🔧 Validación de expresiones más simple

### En el desarrollador
- 📖 Documentación clara para referencia
- 🎓 Fácil aprendizaje (similar a Rust)
- ⚠️ Menos sorpresas (no hay casos extraños)

## 📚 Referencias
- **EBNF Grammar:** `docs/language-design/vela-grammar-ebnf.md`
- **Jira:** [TASK-002](https://velalang.atlassian.net/browse/VELA-566) (subtask de VELA-566)
- **Historia:** [VELA-566](https://velalang.atlassian.net/browse/VELA-566)

## 📝 Lecciones Aprendidas

### ✅ Lo que funcionó bien
1. **Tabla visual** - Facilita comprensión rápida
2. **Ejemplos paso a paso** - Aclaran evaluación
3. **Comparación con otros lenguajes** - Proporciona contexto
4. **Justificación de diseño** - Explica el "por qué"

### ⚠️ Desafíos encontrados
1. **Balance complejidad/simplicidad** - 15 niveles es el punto óptimo
2. **Documentar casos especiales** - Importante prevenir confusión
3. **Asociatividad de exponenciación** - Right es menos común pero más correcto

### 🚀 Próximos pasos
- TASK-003: Definir y categorizar palabras reservadas
- Implementar tabla de precedencia en parser (Phase 1)
- Crear tests de precedencia en compilador

---

**Estado Final:** ✅ COMPLETADA  
**Archivos generados:** 1 (~850 lines)  
**Operadores documentados:** 40+  
**Ejemplos incluidos:** 15+  
**Comparaciones con lenguajes:** 6
