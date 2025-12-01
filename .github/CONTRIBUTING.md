# 🚀 GUÍA DE CONTRIBUCIÓN - PROYECTO VELA

## 📋 ÍNDICE
1. [Introducción](#introducción)
2. [Estructura del Proyecto](#estructura-del-proyecto)
3. [Flujo de Trabajo](#flujo-de-trabajo)
4. [Proceso de Desarrollo](#proceso-de-desarrollo)
5. [Estándares de Calidad](#estándares-de-calidad)
6. [Entregables por Tipo de Tarea](#entregables-por-tipo-de-tarea)

---

## 🎯 INTRODUCCIÓN

Este documento define el proceso completo de desarrollo para el proyecto Vela, desde la planificación hasta la entrega de código productivo.

### **Principios Fundamentales**
1. ✅ **Cada tarea debe generar un entregable tangible**
2. ✅ **Todo código debe estar en control de versiones**
3. ✅ **Toda decisión arquitectónica debe estar documentada**
4. ✅ **Todo cambio debe ser revisado antes de merge**
5. ✅ **Los tests son obligatorios**

---

## 📁 ESTRUCTURA DEL PROYECTO

```
vela/
├── .github/
│   ├── CONTRIBUTING.md          # Este archivo
│   ├── workflows/
│   │   └── ci.yml              # CI/CD pipeline
│   └── ISSUE_TEMPLATE/         # Templates para issues
├── docs/
│   ├── architecture/           # Decisiones arquitectónicas (ADRs)
│   ├── design/                 # Diseños de features
│   ├── api/                    # Documentación de APIs
│   └── user-guides/            # Guías de usuario
├── src/                        # Código fuente
├── tests/                      # Tests automatizados
├── scripts/                    # Scripts de automatización
└── README.md                   # Documentación principal
```

---

## 🔄 FLUJO DE TRABAJO

### **FASE 1: PLANIFICACIÓN (Sprint Planning)**
**Responsable:** Product Owner + Team Lead  
**Entrada:** Backlog priorizado  
**Salida:** Sprint iniciado en Jira

**Acciones:**
1. Identificar Historias de Usuario del Sprint
2. Verificar que cada Historia tenga Subtasks definidas
3. Iniciar Sprint en Jira

---

### **FASE 2: DESARROLLO (Development Cycle)**

#### **PASO 2.1: PREPARACIÓN DE LA HISTORIA**
**Responsable:** GitHub Copilot Agent  
**Entrada:** Historia en estado "Tareas por hacer"  
**Salida:** Rama creada, documentación inicial

**Acciones:**
```bash
# 1. Crear rama para la Historia
git checkout -b feature/US-XXX-descripcion

# 2. Crear estructura de documentación
mkdir -p docs/features/US-XXX
touch docs/features/US-XXX/README.md

# 3. Mover Historia a "En curso" en Jira
```

**Entregables:**
- ✅ Rama Git creada
- ✅ Carpeta de documentación creada
- ✅ Historia en estado "En curso"

---

#### **PASO 2.2: DESARROLLO DE SUBTASKS**

Cada Subtask debe seguir este ciclo:

##### **A) ANÁLISIS Y DISEÑO**

**Entregables según tipo:**

| Tipo de Subtask | Entregable Obligatorio | Ubicación |
|-----------------|------------------------|-----------|
| **Decisión arquitectónica** | ADR (Architecture Decision Record) | `docs/architecture/ADR-XXX-titulo.md` |
| **Diseño de API** | Especificación OpenAPI/Swagger | `docs/api/US-XXX-api-spec.yaml` |
| **Diseño de base de datos** | Diagrama ERD + Migraciones | `docs/design/US-XXX-db-schema.md` |
| **Diseño de interfaz** | Mockups/Wireframes | `docs/design/US-XXX-ui-mockups/` |
| **Investigación técnica** | Documento de investigación | `docs/research/US-XXX-research.md` |

**Template para ADR:**
```markdown
# ADR-XXX: [Título de la Decisión]

## Estado
Aceptado | Propuesto | Rechazado | Obsoleto

## Contexto
[Describe el problema que estamos resolviendo]

## Decisión
[Describe la solución elegida]

## Consecuencias
### Positivas
- [Beneficio 1]
- [Beneficio 2]

### Negativas
- [Trade-off 1]
- [Trade-off 2]

## Alternativas Consideradas
1. [Alternativa 1] - Razón de rechazo
2. [Alternativa 2] - Razón de rechazo

## Referencias
- [Link a documentación]
- [Link a discusión]
```

##### **B) IMPLEMENTACIÓN**

**Entregables según tipo:**

| Tipo de Subtask | Entregable Obligatorio | Ubicación |
|-----------------|------------------------|-----------|
| **Feature nueva** | Código + Tests unitarios | `src/` + `tests/unit/` |
| **API endpoint** | Código + Tests integración | `src/api/` + `tests/integration/` |
| **Componente UI** | Código + Storybook/Tests visuales | `src/components/` + `stories/` |
| **Refactoring** | Código + Tests de regresión | `src/` + `tests/` |
| **Bug fix** | Código + Test que reproduce el bug | `src/` + `tests/` |
| **Configuración** | Archivos de config + Documentación | `config/` + `docs/` |

**Reglas de implementación:**
- ✅ Todo código debe tener tests (cobertura mínima 80%)
- ✅ Todo código debe pasar el linter
- ✅ Todo código debe estar documentado (JSDoc/docstrings)
- ✅ Commits deben seguir Conventional Commits

**Formato de commits:**
```
tipo(scope): descripción breve

- Detalle 1
- Detalle 2

Refs: VELA-XXX
```

Tipos válidos: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

##### **C) DOCUMENTACIÓN**

**Entregables obligatorios por Subtask:**

1. **README de la feature** (`docs/features/US-XXX/TASK-XXX.md`)
```markdown
# TASK-XXX: [Título]

## Objetivo
[Qué problema resuelve]

## Implementación
[Cómo se resolvió]

## Archivos modificados
- `src/file1.ts` - [Descripción de cambios]
- `src/file2.ts` - [Descripción de cambios]

## Tests
- `tests/unit/test1.spec.ts` - [Qué testea]
- `tests/integration/test2.spec.ts` - [Qué testea]

## Cómo usar
[Ejemplos de uso]

## Referencias
- ADR-XXX
- Issue VELA-XXX
```

2. **Actualización de CHANGELOG** (`CHANGELOG.md`)
```markdown
## [Unreleased]

### Added
- [TASK-XXX] Descripción de la funcionalidad

### Changed
- [TASK-XXX] Descripción del cambio

### Fixed
- [TASK-XXX] Descripción del fix
```

##### **D) REVISIÓN Y COMMIT**

**Acciones:**
```bash
# 1. Ejecutar tests
npm test  # o pytest, cargo test, etc.

# 2. Ejecutar linter
npm run lint

# 3. Verificar cobertura
npm run coverage

# 4. Commit con mensaje descriptivo
git add .
git commit -m "feat(US-XXX): implementar TASK-XXX

- Implementación de [funcionalidad]
- Tests unitarios con 95% cobertura
- Documentación en docs/features/US-XXX/

Refs: VELA-XXX"

# 5. Actualizar estado en Jira
# Subtask: En curso -> Finalizada
```

**Checklist antes de marcar Subtask como Finalizada:**
- [ ] ✅ Código implementado y funcional
- [ ] ✅ Tests escritos y pasando (cobertura >= 80%)
- [ ] ✅ Documentación creada/actualizada
- [ ] ✅ Linter sin errores
- [ ] ✅ Commit realizado con mensaje descriptivo
- [ ] ✅ Subtask en estado "Finalizada" en Jira

---

#### **PASO 2.3: COMPLETAR HISTORIA DE USUARIO**

Cuando todas las Subtasks están finalizadas:

**Acciones:**
```bash
# 1. Crear Pull Request
git push origin feature/US-XXX-descripcion

# 2. Crear PR en GitHub con template
# Título: [US-XXX] Descripción de la Historia
```

**Template de Pull Request:**
```markdown
## 📋 Descripción
[Resumen de la Historia de Usuario]

## 🎯 Objetivos
- [ ] Objetivo 1
- [ ] Objetivo 2

## 🔨 Cambios realizados

### Subtasks completadas
- [x] TASK-XXX: [Descripción]
- [x] TASK-YYY: [Descripción]

### Archivos principales modificados
- `src/file1.ts` - [Descripción]
- `src/file2.ts` - [Descripción]

## 📚 Documentación
- ADR-XXX: [Título]
- docs/features/US-XXX/README.md

## ✅ Tests
- Tests unitarios: XXX pasando
- Tests integración: XXX pasando
- Cobertura: XX%

## 🎬 Cómo probar
1. [Paso 1]
2. [Paso 2]
3. Verificar [resultado esperado]

## 📸 Screenshots (si aplica)
[Capturas de pantalla]

## ⚠️ Breaking Changes
[Si hay cambios que rompen compatibilidad]

## 🔗 Referencias
- Jira: VELA-XXX
- Diseño: [Link]
- Discusiones: [Link]

## ✅ Checklist
- [ ] Código revisado y funcional
- [ ] Tests pasando (cobertura >= 80%)
- [ ] Documentación completa
- [ ] Sin errores de linting
- [ ] CHANGELOG actualizado
- [ ] ADRs creados (si aplica)
```

**Entregables de la Historia:**
- ✅ Pull Request creada
- ✅ Código completo con tests
- ✅ Documentación completa
- ✅ CHANGELOG actualizado
- ✅ Historia en estado "En revisión"

---

#### **PASO 2.4: CODE REVIEW**

**Responsable:** Tech Lead / Senior Developer  
**Entrada:** Pull Request creada  
**Salida:** PR aprobada o cambios solicitados

**Checklist de revisión:**
- [ ] ✅ El código cumple con los estándares del proyecto
- [ ] ✅ Los tests son adecuados y están pasando
- [ ] ✅ La documentación es clara y completa
- [ ] ✅ No hay código comentado o debug innecesario
- [ ] ✅ Las decisiones arquitectónicas están justificadas (ADRs)
- [ ] ✅ El código es mantenible y legible
- [ ] ✅ No hay vulnerabilidades de seguridad
- [ ] ✅ El rendimiento es aceptable

**Acciones:**
```bash
# Si hay cambios solicitados:
# - Implementar cambios
# - Commit y push
# - Solicitar nueva revisión

# Si está aprobada:
# - Merge a main/develop
# - Mover Historia a "Finalizada" en Jira
# - Eliminar rama feature
```

---

### **FASE 3: CIERRE DE SPRINT (Sprint Closure)**

Cuando todas las Historias del Sprint están finalizadas:

**Responsable:** GitHub Copilot Agent  
**Entrada:** Todas las Historias en "Finalizada"  
**Salida:** Sprint cerrado, release notes generadas

**Acciones:**
1. Verificar que todas las PRs están merged
2. Generar Release Notes
3. Crear tag de versión
4. Cerrar Sprint en Jira
5. Deploy a staging/producción (según aplique)

**Template de Release Notes** (`docs/releases/sprint-XX.md`):
```markdown
# 🚀 Sprint XX - Release Notes

**Fecha:** [Fecha de inicio] - [Fecha de cierre]  
**Versión:** vX.Y.Z

## 📊 Resumen del Sprint
- **Historias completadas:** XX
- **Subtasks completadas:** XXX
- **Commits:** XXX
- **Tests agregados:** XXX

## ✨ Nuevas Features
### [US-XXX] Título de la Historia
[Descripción breve]
- TASK-XXX: [Descripción]
- TASK-YYY: [Descripción]

**Documentación:** [Link a docs]

## 🔧 Mejoras
[Lista de mejoras]

## 🐛 Bugs Corregidos
[Lista de bugs]

## 📚 Documentación Agregada
- ADR-XXX: [Título]
- [Otra documentación]

## 🔄 Cambios Técnicos
[Cambios en arquitectura, dependencias, etc.]

## ⚠️ Breaking Changes
[Cambios que afectan compatibilidad]

## 🎯 Próximo Sprint
[Preview de lo que viene]
```

---

## 📏 ESTÁNDARES DE CALIDAD

### **Código**
- ✅ Cobertura de tests >= 80%
- ✅ Sin errores de linting
- ✅ Sin vulnerabilidades críticas (npm audit, Snyk)
- ✅ Complejidad ciclomática <= 10

### **Documentación**
- ✅ Cada feature documentada en `docs/features/`
- ✅ Decisiones arquitectónicas en ADRs
- ✅ APIs documentadas en OpenAPI/Swagger
- ✅ README actualizado

### **Tests**
- ✅ Tests unitarios para lógica de negocio
- ✅ Tests de integración para APIs
- ✅ Tests end-to-end para flujos críticos
- ✅ Tests de rendimiento para operaciones pesadas

---

## 🎯 ENTREGABLES POR TIPO DE TAREA

### **Epic**
- ✅ Documento de visión (Product Requirements Document)
- ✅ Arquitectura de alto nivel
- ✅ Plan de implementación por fases
- ✅ Métricas de éxito

### **Historia de Usuario**
- ✅ Pull Request merged
- ✅ Código en main/develop
- ✅ Tests pasando
- ✅ Documentación completa
- ✅ Release notes

### **Subtask - Decisión Arquitectónica**
- ✅ ADR documentado
- ✅ Diagrama de arquitectura (si aplica)
- ✅ Commit con decisión implementada

### **Subtask - Implementación**
- ✅ Código funcional
- ✅ Tests unitarios (>= 80% cobertura)
- ✅ Documentación inline (docstrings/JSDoc)
- ✅ Commit con mensaje descriptivo

### **Subtask - Testing**
- ✅ Tests implementados
- ✅ Reporte de cobertura
- ✅ Commit con tests

### **Subtask - Documentación**
- ✅ Documento markdown en `docs/`
- ✅ Diagramas (si aplica)
- ✅ Ejemplos de uso
- ✅ Commit con documentación

---

## 🤖 AUTOMATIZACIÓN CON GITHUB COPILOT

El agente de GitHub Copilot ejecutará este workflow automáticamente:

```python
# Pseudocódigo del proceso automatizado

for sprint in sprints:
    # Fase 1: Iniciar Sprint (manual por Product Owner)
    wait_for_sprint_start(sprint)
    
    # Fase 2: Desarrollar cada Historia
    for historia in sprint.historias:
        # Crear rama
        git_create_branch(f"feature/{historia.key}")
        
        # Procesar Subtasks
        for subtask in historia.subtasks:
            # Mover a "En curso"
            jira.transition(subtask, "En curso")
            
            # Generar entregables según tipo
            deliverables = generate_deliverables(subtask)
            
            # Implementar código
            code = implement_code(subtask)
            
            # Crear tests
            tests = create_tests(subtask)
            
            # Documentar
            docs = create_documentation(subtask)
            
            # Commit
            git.commit(code, tests, docs, deliverables)
            
            # Mover a "Finalizada"
            jira.transition(subtask, "Finalizada")
        
        # Crear Pull Request
        pr = github.create_pull_request(historia)
        
        # Mover Historia a "En revisión"
        jira.transition(historia, "En revisión")
        
        # Esperar aprobación (manual)
        wait_for_approval(pr)
        
        # Merge
        github.merge(pr)
        
        # Mover a "Finalizada"
        jira.transition(historia, "Finalizada")
    
    # Fase 3: Cerrar Sprint
    release_notes = generate_release_notes(sprint)
    git.tag(f"sprint-{sprint.number}")
    jira.close_sprint(sprint)
```

---

## 📞 CONTACTO Y SOPORTE

Para preguntas sobre este proceso:
- **Tech Lead:** [Nombre]
- **Product Owner:** [Nombre]
- **Documentación:** `docs/`
- **Issues:** GitHub Issues

---

**Última actualización:** 2025-11-30  
**Versión:** 1.0.0
