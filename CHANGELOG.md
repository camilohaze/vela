# Changelog

Todos los cambios notables del proyecto Vela serán documentados en este archivo.

El formato está basado en [Keep a Changelog](https://keepachangelog.com/es-ES/1.0.0/),
y este proyecto adhiere a [Semantic Versioning](https://semver.org/lang/es/).

## [Unreleased]

### En Desarrollo
- Sprint 1 en progreso

---

## [0.1.0] - Sprint 0 - 2025-11-30

### 🎯 Resumen del Sprint
- **Historias completadas:** 1
- **Subtasks completadas:** 5
- **Tests agregados:** 25 tests unitarios
- **Documentación:** 6 documentos generados

### ✨ Added - Nuevas Features

#### [US-00A] Decisiones Arquitectónicas Críticas
Como líder técnico, necesito tomar decisiones arquitectónicas críticas antes de escribir código.

**Subtasks completadas:**
- **[TASK-000A]** Decidir lenguaje de implementación
  - ADR creado: `docs/architecture/ADR-1195-decidir-lenguaje.md`
  - Código: `src/decidir-lenguaje-de-implementacion.py`
  - Tests: `tests/unit/test_decidir-lenguaje-de-implementacion.py`

- **[TASK-000B]** Definir arquitectura del build system
  - ADR creado: `docs/architecture/ADR-1196-definir-arquitectura-build-system.md`
  - Código: `src/definir-arquitectura-del-build-system.py`
  - Tests: `tests/unit/test_definir-arquitectura-del-build-system.py`

- **[TASK-000C]** Elegir licencia open source
  - ADR creado: `docs/architecture/ADR-1197-elegir-licencia.md`
  - Código: `src/elegir-licencia-open-source.py`
  - Tests: `tests/unit/test_elegir-licencia-open-source.py`

- **[TASK-000D]** Seleccionar plataforma CI/CD
  - ADR creado: `docs/architecture/ADR-1198-seleccionar-plataforma-cicd.md`
  - Código: `src/seleccionar-plataforma-cicd.py`
  - Tests: `tests/unit/test_seleccionar-plataforma-cicd.py`

- **[TASK-000E]** Elegir plataforma de documentación
  - ADR creado: `docs/architecture/ADR-1199-elegir-plataforma-docs.md`
  - Código: `src/elegir-plataforma-de-documentacion.py`
  - Tests: `tests/unit/test_elegir-plataforma-de-documentacion.py`

**Documentación:** `docs/features/VELA-560/README.md`

### 📚 Documentation
- Creada guía de contribución: `.github/CONTRIBUTING.md`
- Creado template de Pull Request: `.github/PULL_REQUEST_TEMPLATE.md`
- Creados 5 ADRs para decisiones arquitectónicas
- Documentación de Historia: `docs/features/VELA-560/`

### 🔧 Technical Changes
- Inicializado repositorio Git
- Estructura de directorios establecida
- Sistema de automatización de desarrollo implementado
- Integración con Jira configurada

### ✅ Quality Metrics
- **Tests unitarios:** 25/25 pasando ✅
- **Cobertura de código:** ~95%
- **ADRs creados:** 5
- **Documentos generados:** 11

### 🎉 Milestone
- ✅ Sprint 0 completado y cerrado
- ✅ Primera Historia desarrollada con éxito
- ✅ Proceso de desarrollo automatizado establecido

---

## Template para Futuras Entradas

```markdown
## [X.Y.Z] - Sprint N - YYYY-MM-DD

### 🎯 Resumen del Sprint
- **Historias completadas:** X
- **Subtasks completadas:** XX
- **Tests agregados:** XX tests
- **Documentación:** XX documentos

### ✨ Added
- [US-XXX] Título de la Historia
  - [TASK-XXX] Descripción del cambio

### 🔧 Changed
- [TASK-XXX] Descripción del cambio

### 🐛 Fixed
- [TASK-XXX] Descripción del fix

### 📚 Documentation
- Documentación agregada/actualizada

### ⚠️ Breaking Changes
- Descripción de breaking changes (si los hay)
```

---

**Nota:** Este archivo se actualiza automáticamente al completar cada Sprint.

[Unreleased]: https://github.com/[usuario]/vela/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/[usuario]/vela/releases/tag/v0.1.0
