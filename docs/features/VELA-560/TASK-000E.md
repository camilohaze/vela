# TASK-000E: Elegir plataforma de documentación

## 📋 Información General
- **Historia:** VELA-560 (US-00A)
- **Subtask:** VELA-1199
- **Estado:** Completada ✅
- **Fecha:** 2025-11-30

## 🎯 Objetivo
Seleccionar la plataforma y herramientas para documentación técnica, guías de usuario, referencia del lenguaje y documentación de desarrollo del proyecto Vela.

## 🔨 Implementación

### Decisión: rustdoc + mdBook + GitHub Pages

Se decidió utilizar **enfoque híbrido**:

1. **rustdoc**: Documentación API generada desde código
2. **mdBook**: Guías, tutoriales, referencia del lenguaje
3. **GitHub Pages**: Hosting gratuito

### Estructura de documentación

```
docs/
├── architecture/       # ADRs (GitHub repo)
├── features/           # Docs por Historia (GitHub repo)
├── api/               # Specs OpenAPI (GitHub repo)
├── design/            # Diseños (GitHub repo)
└── book/              # mdBook source
    ├── src/
    │   ├── SUMMARY.md
    │   ├── getting-started.md
    │   ├── language-reference/
    │   ├── tutorials/
    │   └── cookbook/
    └── book.toml

# Hosting:
# https://velalang.github.io/vela/
# ├── /api/     → rustdoc
# └── /book/    → mdBook
```

### Archivos generados

- **ADR**: `docs/architecture/ADR-005-plataforma-documentacion.md`
- **Config example**: `book.toml` incluido en ADR
- **Workflow**: Especificado en ADR (a crear en futuros sprints)

### Rationale

**¿Por qué rustdoc?**
- Generación automática desde comentarios de código
- Mantiene docs sincronizadas con código
- Estándar en Rust ecosystem
- Ejemplos ejecutables con doctests

**¿Por qué mdBook?**
- Diseñado para The Rust Book
- Perfecto para tutoriales largos
- Búsqueda integrada
- Markdown simple
- Temas personalizables

**¿Por qué GitHub Pages?**
- Hosting gratuito
- Integrado con GitHub Actions
- Custom domain posible
- HTTPS automático

### Ejemplos de uso

**rustdoc:**
```rust
/// Parse a Vela source file
///
/// # Examples
///
/// ```
/// use vela_parser::parse;
/// let ast = parse("let x = 42;");
/// assert!(ast.is_ok());
/// ```
pub fn parse(source: &str) -> Result<Ast, ParseError> {
    // ...
}
```

**mdBook:**
```markdown
# Getting Started

## Installing Vela

Download the latest release...

## Hello World

Create `hello.vela`:
\`\`\`vela
fn main() {
    print("Hello, Vela!");
}
\`\`\`
```

## ✅ Criterios de Aceptación

- [x] ADR-005 creado con arquitectura de docs
- [x] rustdoc como herramienta para API docs
- [x] mdBook como herramienta para guías
- [x] GitHub Pages como plataforma de hosting
- [x] Estructura de directorios definida
- [x] Configuración de book.toml especificada
- [x] Workflow de deploy especificado
- [x] Comparación con alternativas

## 📊 Métricas

- **Archivos creados**: 1
  - 1 ADR (incluye config y workflow)
- **Herramientas seleccionadas**: 3
  - rustdoc
  - mdBook
  - GitHub Pages
- **Alternativas evaluadas**: 5 (Docusaurus, Sphinx, GitBook, ReadTheDocs, solo rustdoc)

## 🔗 Referencias

- **Jira**: [VELA-1199](https://velalang.atlassian.net/browse/VELA-1199)
- **Historia**: [VELA-560](https://velalang.atlassian.net/browse/VELA-560)
- **ADR**: `docs/architecture/ADR-005-plataforma-documentacion.md`
- **rustdoc**: https://doc.rust-lang.org/rustdoc/
- **mdBook**: https://rust-lang.github.io/mdBook/
- **The Rust Book**: https://doc.rust-lang.org/book/
- **GitHub Pages**: https://pages.github.com/

---

*Completada: Sprint 0 - 2025-11-30*
