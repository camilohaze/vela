# ADR-005: Elegir Plataforma de Documentación

## Estado
✅ Aceptado

## Fecha
2025-11-30

## Contexto

El proyecto Vela necesita una plataforma de documentación robusta para:

- **Documentación técnica**: API, arquitectura, diseño
- **Guías de usuario**: Tutoriales, ejemplos, cookbook
- **Referencia del lenguaje**: Sintaxis, stdlib, buenas prácticas
- **Documentación de desarrollo**: Cómo contribuir, build, testing

Requisitos:
- Generación automática desde código (rustdoc)
- Versionado de documentación
- Búsqueda integrada
- Hosting gratuito
- Markdown support
- Temas personalizables

## Decisión

**Se decide utilizar un enfoque híbrido:**

1. **rustdoc + GitHub Pages** para documentación API
2. **mdBook** para guías y tutoriales
3. **docs.rs** para publicación de crates

### Estructura:

```
docs/
├── architecture/       # ADRs (GitHub repo)
├── features/           # Documentación por Historia (GitHub repo)
├── api/               # Specs OpenAPI (GitHub repo)
├── design/            # Diseños técnicos (GitHub repo)
└── book/              # mdBook source
    ├── src/
    │   ├── SUMMARY.md
    │   ├── getting-started.md
    │   ├── language-reference/
    │   ├── tutorials/
    │   └── cookbook/
    └── book.toml
```

### Hosting:
- **GitHub Pages**: `https://velalang.github.io/vela/`
  - `/api/` → rustdoc output
  - `/book/` → mdBook output

## Consecuencias

### Positivas

- ✅ **rustdoc**: Generación automática desde comentarios de código
- ✅ **mdBook**: Excelente para tutoriales y guías narrativas
- ✅ **GitHub Pages**: Hosting gratuito, integrado con GitHub Actions
- ✅ **docs.rs**: Publicación automática al publicar en crates.io
- ✅ Búsqueda integrada en mdBook
- ✅ Markdown familiar para contribuidores
- ✅ Versionado con Git

### Negativas

- ⚠️ Requiere mantener dos formatos (rustdoc + mdBook)
- ⚠️ Setup inicial de GitHub Pages
- ⚠️ Necesita configuración de CI para deploy automático

## Alternativas Consideradas

### 1. Solo rustdoc
**Rechazada porque:**
- No es adecuado para tutoriales largos
- Limitado a documentación de API
- Menos flexible para contenido narrativo

### 2. Docusaurus / VitePress
**Rechazada porque:**
- Requiere Node.js (dependencia extra)
- Más complejo de configurar
- Overkill para proyecto inicial
- No tan adoptado en comunidad Rust

### 3. Sphinx
**Rechazada porque:**
- Requiere Python (dependencia extra)
- Más orientado a Python
- Menos adopción en Rust

### 4. GitBook
**Rechazada porque:**
- Plan gratuito limitado
- Menos control sobre hosting
- No tan integrado con Rust ecosystem

### 5. ReadTheDocs
**Rechazada porque:**
- Más orientado a Python
- Menos control sobre build process
- GitHub Pages es suficiente

## Referencias

- **Jira**: VELA-1199 (TASK-000E)
- **Historia**: VELA-560 (US-00A)
- **rustdoc**: https://doc.rust-lang.org/rustdoc/
- **mdBook**: https://rust-lang.github.io/mdBook/
- **Ejemplo**: https://doc.rust-lang.org/book/

## Implementación

### mdBook configuration:

```toml
# docs/book/book.toml

[book]
title = "The Vela Programming Language"
authors = ["Vela Contributors"]
language = "en"
multilingual = false
src = "src"

[build]
build-dir = "../../target/book"

[output.html]
default-theme = "rust"
preferred-dark-theme = "navy"
git-repository-url = "https://github.com/velalang/vela"
edit-url-template = "https://github.com/velalang/vela/edit/main/docs/book/{path}"

[output.html.search]
enable = true
```

### GitHub Actions para deploy:

```yaml
# .github/workflows/docs.yml

name: Deploy Documentation

on:
  push:
    branches: [ main ]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install mdBook
        run: |
          cargo install mdbook
      
      - name: Build rustdoc
        run: cargo doc --no-deps
      
      - name: Build mdBook
        run: cd docs/book && mdbook build
      
      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./target
```

Ver implementación en: `.github/workflows/docs.yml` (a crear en futuras Historias)

---

*ADR creado: 2025-11-30*  
*Sprint: 0*
