# TASK-000C: Elegir licencia open source

## 📋 Información General
- **Historia:** VELA-560 (US-00A)
- **Subtask:** VELA-1197
- **Estado:** Completada ✅
- **Fecha:** 2025-11-30

## 🎯 Objetivo
Seleccionar la licencia open source para el proyecto Vela, equilibrando apertura, adopción empresarial y protección de patentes.

## 🔨 Implementación

### Decisión: MIT OR Apache-2.0 (Dual License)

Se decidió utilizar **licencia dual** siguiendo el modelo de Rust:

- **MIT License**: Máxima permisividad y simplicidad
- **Apache License 2.0**: Protección explícita de patentes

### Archivos generados

- **ADR**: `docs/architecture/ADR-003-licencia-open-source.md`
- **Licencias**: 
  - `LICENSE-MIT`
  - `LICENSE-APACHE`
- **Header para código**: Especificado en ADR

### Rationale

**¿Por qué licencia dual?**

1. **MIT**: Simple, ampliamente entendida, compatible con casi todo
2. **Apache 2.0**: Protección de patentes, favorecida por empresas
3. **Dual**: Usuarios eligen la que mejor se adapte a sus necesidades

**Modelo probado:**
- Rust (lenguaje): MIT OR Apache-2.0
- Tokio (runtime): MIT OR Apache-2.0
- Serde (serialization): MIT OR Apache-2.0

### Headers de código

Todos los archivos fuente deben incluir:

```rust
// Copyright (c) 2025 Vela Contributors
// SPDX-License-Identifier: MIT OR Apache-2.0
```

### Uso en proyectos

Los usuarios pueden elegir:

```toml
# Opción 1: Usar bajo MIT
[dependencies]
vela = { version = "0.1", license = "MIT" }

# Opción 2: Usar bajo Apache-2.0
[dependencies]
vela = { version = "0.1", license = "Apache-2.0" }
```

## ✅ Criterios de Aceptación

- [x] ADR-003 creado con justificación legal
- [x] `LICENSE-MIT` creado con texto completo
- [x] `LICENSE-APACHE` creado con texto completo
- [x] Headers SPDX definidos para archivos de código
- [x] Documentación de cómo aplicar licencias
- [x] Comparación con alternativas (GPL, BSD, single license)

## 📊 Métricas

- **Archivos creados**: 3
  - 1 ADR
  - 2 archivos de licencia
- **Alternativas evaluadas**: 4 (GPL v3, BSD 3-Clause, MIT only, Apache-2.0 only)

## 🔗 Referencias

- **Jira**: [VELA-1197](https://velalang.atlassian.net/browse/VELA-1197)
- **Historia**: [VELA-560](https://velalang.atlassian.net/browse/VELA-560)
- **ADR**: `docs/architecture/ADR-003-licencia-open-source.md`
- **MIT License**: https://opensource.org/licenses/MIT
- **Apache 2.0**: https://www.apache.org/licenses/LICENSE-2.0
- **SPDX**: https://spdx.org/licenses/

---

*Completada: Sprint 0 - 2025-11-30*
