# ADR-003: Elegir Licencia Open Source

## Estado
✅ Aceptado

## Fecha
2025-11-30

## Contexto

Vela es un proyecto de código abierto que necesita una licencia clara para:

- Proteger los derechos de los contribuidores
- Definir términos de uso y redistribución
- Fomentar adopción comercial y no comercial
- Establecer responsabilidades legales
- Facilitar contribuciones de la comunidad

Consideraciones importantes:
- Vela es un lenguaje de programación (toolchain)
- Queremos máxima adopción
- Necesitamos protección de patentes
- Queremos permitir uso comercial

## Decisión

**Se decide usar licencia dual: MIT OR Apache-2.0**

Esta es la misma estrategia que Rust, y permite:

1. **Licencia MIT**: Permisiva, simple, ampliamente reconocida
2. **Licencia Apache 2.0**: Incluye protección explícita de patentes

Los usuarios pueden elegir cualquiera de las dos licencias.

## Consecuencias

### Positivas

- ✅ Máxima permisividad para adopción comercial
- ✅ Protección de patentes (Apache 2.0)
- ✅ Simplicidad legal (MIT)
- ✅ Compatibilidad con la mayoría de proyectos
- ✅ Mismo modelo que Rust (familiar para la comunidad)
- ✅ Permite inclusión en proyectos propietarios

### Negativas

- ⚠️ No hay copyleft (alguien podría hacer fork propietario)
- ⚠️ Requiere incluir dos archivos de licencia
- ⚠️ Usuarios deben elegir cuál licencia seguir

## Alternativas Consideradas

### 1. GPL v3 (Copyleft)
**Rechazada porque:**
- Restringe uso en software propietario
- Menor adopción corporativa
- No apropiada para toolchains (compiladores deben ser permisivos)

### 2. BSD 3-Clause
**Rechazada porque:**
- No incluye protección de patentes
- Cláusula de no-endorsement es redundante

### 3. Solo MIT
**Rechazada porque:**
- No incluye protección explícita de patentes
- Apache 2.0 es más defensivo legalmente

### 4. Solo Apache 2.0
**Rechazada porque:**
- Más compleja que MIT
- Algunos proyectos prefieren MIT por simplicidad

## Referencias

- **Jira**: VELA-1197 (TASK-000C)
- **Historia**: VELA-560 (US-00A)
- **Licencia MIT**: https://opensource.org/licenses/MIT
- **Licencia Apache 2.0**: https://www.apache.org/licenses/LICENSE-2.0
- **Rust licensing**: https://github.com/rust-lang/rust#license

## Implementación

```toml
# Cargo.toml - Declaración de licencia

[package]
license = "MIT OR Apache-2.0"
```

Archivos a crear:
- `LICENSE-MIT` - Texto completo de licencia MIT
- `LICENSE-APACHE` - Texto completo de licencia Apache 2.0
- `README.md` - Sección de licencia

### Header en archivos de código:

```rust
// Copyright 2025 Vela Contributors
// Licensed under the MIT OR Apache-2.0 license
// See LICENSE-MIT or LICENSE-APACHE files for details
```

---

*ADR creado: 2025-11-30*  
*Sprint: 0*
