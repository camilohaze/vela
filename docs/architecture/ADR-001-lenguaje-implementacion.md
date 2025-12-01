# ADR-001: Decidir Lenguaje de Implementación para Vela

## Estado
✅ Aceptado

## Fecha
2025-11-30

## Contexto

Vela es un lenguaje de programación moderno que necesita ser implementado. La decisión del lenguaje de implementación es crítica porque:

- Define el ecosistema de herramientas disponibles
- Impacta el rendimiento del compilador
- Determina la facilidad de contribución
- Afecta la portabilidad del compilador

## Decisión

**Se decide implementar Vela en Rust.**

### Razones principales:

1. **Rendimiento**: Rust ofrece rendimiento comparable a C/C++ sin garbage collector
2. **Seguridad de memoria**: El sistema de ownership elimina errores comunes de memoria
3. **Herramientas modernas**: Cargo, rustfmt, clippy proporcionan excelente DX
4. **Comunidad activa**: Gran ecosistema de librerías para compiladores (LLVM bindings, parser generators)
5. **Portabilidad**: Rust compila a múltiples plataformas sin modificaciones

## Consecuencias

### Positivas

- ✅ Compilador rápido y eficiente
- ✅ Menos bugs de memoria y seguridad
- ✅ Excelente tooling (cargo, rustfmt, clippy)
- ✅ Fácil integración con LLVM
- ✅ Comunidad activa de compiladores en Rust (rustc, swc, deno)

### Negativas

- ⚠️ Curva de aprendizaje inicial para el equipo
- ⚠️ Tiempos de compilación más largos que lenguajes dinámicos
- ⚠️ Borrow checker puede ser restrictivo inicialmente

## Alternativas Consideradas

### 1. C++
**Rechazada porque:**
- Gestión manual de memoria propensa a errores
- Tooling menos moderno
- Mayor complejidad de sintaxis
- Ecosistema fragmentado (múltiples build systems)

### 2. Go
**Rechazada porque:**
- Garbage collector (no deseable para un compilador de alto rendimiento)
- Menos expresivo para compiladores (sin generics hasta recientemente)
- Menos control sobre optimizaciones

### 3. OCaml/Haskell
**Rechazada porque:**
- Menor adopción en la comunidad de sistemas
- Menos herramientas modernas
- Interoperabilidad limitada con C/LLVM

### 4. Python/JavaScript
**Rechazada porque:**
- Rendimiento insuficiente para un compilador productivo
- No apropiado para toolchain de sistemas

## Referencias

- **Jira**: VELA-1195 (TASK-000A)
- **Historia**: VELA-560 (US-00A)
- **Ejemplos de compiladores en Rust**: 
  - rustc (compilador de Rust)
  - swc (compilador de JavaScript/TypeScript)
  - deno (runtime de JavaScript)

## Implementación

```rust
// src/main.rs - Estructura inicial del compilador

fn main() {
    println!("Vela Compiler v0.1.0");
    println!("Implementado en Rust");
}
```

Ver código en: `src/main.rs` (a desarrollar en futuras Historias)

---

*ADR creado: 2025-11-30*  
*Sprint: 0*
