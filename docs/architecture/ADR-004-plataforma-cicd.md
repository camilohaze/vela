# ADR-004: Seleccionar Plataforma CI/CD

## Estado
✅ Aceptado

## Fecha
2025-11-30

## Contexto

El proyecto Vela necesita integración continua y despliegue continuo (CI/CD) para:

- Ejecutar tests automáticamente en cada commit
- Verificar builds en múltiples plataformas (Linux, macOS, Windows)
- Ejecutar linters y formatters
- Generar releases automáticamente
- Publicar documentación
- Mantener calidad de código

Requisitos:
- Integración nativa con GitHub
- Soporte para múltiples plataformas
- Gratuito para proyectos open source
- Fácil configuración
- Cache de dependencias

## Decisión

**Se decide utilizar GitHub Actions como plataforma CI/CD principal.**

### Workflows a implementar:

1. **CI (Continuous Integration)**:
   - Build en Linux, macOS, Windows
   - Tests unitarios e integración
   - Linting (clippy)
   - Formatting (rustfmt)
   - Coverage de código

2. **Release**:
   - Generación de binarios para cada plataforma
   - Publicación en GitHub Releases
   - Actualización de changelog

3. **Documentation**:
   - Generación de docs con `cargo doc`
   - Deploy a GitHub Pages

## Consecuencias

### Positivas

- ✅ Integración nativa con GitHub (no requiere configuración externa)
- ✅ Gratuito para proyectos públicos (ilimitado)
- ✅ Soporte nativo para matrices (múltiples OS/versiones)
- ✅ Cache integrado para dependencias
- ✅ Marketplace con miles de actions reutilizables
- ✅ Secrets management integrado
- ✅ Excelente documentación y comunidad

### Negativas

- ⚠️ Vendor lock-in con GitHub
- ⚠️ Runners compartidos pueden ser lentos en peak times
- ⚠️ Límites de tiempo de ejecución (6 horas por job)

## Alternativas Consideradas

### 1. GitLab CI/CD
**Rechazada porque:**
- Requiere migrar de GitHub o setup adicional
- Menos adoption en comunidad Rust
- Configuración más compleja

### 2. Travis CI
**Rechazada porque:**
- Ya no es gratuito para open source
- Menos features que GitHub Actions
- Menor integración con GitHub

### 3. CircleCI
**Rechazada porque:**
- Límites más restrictivos en plan gratuito
- Requiere configuración externa
- Menos adoption

### 4. Self-hosted (Jenkins, etc.)
**Rechazada porque:**
- Requiere infraestructura y mantenimiento
- Costos de hosting
- Overkill para el proyecto actual

## Referencias

- **Jira**: VELA-1198 (TASK-000D)
- **Historia**: VELA-560 (US-00A)
- **GitHub Actions**: https://docs.github.com/en/actions
- **Rust CI example**: https://github.com/actions-rs/example

## Implementación

```yaml
# .github/workflows/ci.yml

name: CI

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

env:
  CARGO_TERM_COLOR: always

jobs:
  test:
    name: Test
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          profile: minimal
          override: true
      
      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Build
        run: cargo build --verbose
      
      - name: Run tests
        run: cargo test --verbose
      
      - name: Check formatting
        run: cargo fmt -- --check
      
      - name: Run clippy
        run: cargo clippy -- -D warnings

  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - uses: actions-rs/tarpaulin@v0.1
        with:
          args: '--ignore-tests'
      - name: Upload to codecov.io
        uses: codecov/codecov-action@v3
```

Ver implementación en: `.github/workflows/` (a crear en futuras Historias)

---

*ADR creado: 2025-11-30*  
*Sprint: 0*
