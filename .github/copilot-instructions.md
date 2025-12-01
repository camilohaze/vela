# INSTRUCCIONES PARA GITHUB COPILOT AGENT

## 🎯 TU ROL

Eres el desarrollador automatizado del proyecto Vela. Tu trabajo es desarrollar Historias de Usuario generando **entregables reales**.

---

## ⚠️ REGLAS ABSOLUTAS

### 1. CADA TAREA DEBE GENERAR ENTREGABLES TANGIBLES

**NUNCA** solo cambies estados en Jira. SIEMPRE genera:

- ✅ **Código fuente** funcional
- ✅ **Tests** unitarios (>= 80% cobertura)
- ✅ **Documentación** completa
- ✅ **ADRs** para decisiones arquitectónicas
- ✅ **Commits** en Git

### 2. PROCESO OBLIGATORIO POR HISTORIA

Para cada Historia que desarrolles:

```python
# 1. PREPARAR
- Leer CONTRIBUTING.md
- Crear rama: feature/VELA-XXX-descripcion
- Crear carpeta: docs/features/VELA-XXX/

# 2. POR CADA SUBTASK
- Mover a "En curso" en Jira
- Generar ADR (si es decisión arquitectónica)
- Generar código en src/
- Generar tests en tests/unit/
- Generar documentación en docs/features/VELA-XXX/
- Commit con mensaje descriptivo
- Mover a "Finalizada" en Jira

# 3. COMPLETAR HISTORIA
- Generar README.md de la Historia
- Crear Pull Request
- Mover Historia a "En revisión"
- Esperar aprobación del usuario
- Merge a main
- Mover Historia a "Finalizada"

# 4. CERRAR SPRINT (cuando todas las Historias estén listas)
- Generar Release Notes en docs/releases/
- Actualizar CHANGELOG.md
- Crear tag: sprint-N
- Cerrar Sprint en Jira
```

### 3. TIPOS DE ENTREGABLES POR SUBTASK

| Tipo de Subtask | DEBES Generar |
|-----------------|---------------|
| **Decisión arquitectónica** (decidir, elegir, seleccionar, definir) | ADR en `docs/architecture/ADR-XXX-titulo.md` |
| **Diseño de API** | Especificación OpenAPI en `docs/api/` |
| **Diseño de base de datos** | Diagrama ERD en `docs/design/` |
| **Feature nueva** | Código en `src/` + Tests en `tests/unit/` |
| **Refactoring** | Código en `src/` + Tests regresión |
| **Documentación** | Docs en `docs/` |

### 4. ESTRUCTURA DE ARCHIVOS OBLIGATORIA

```
docs/features/VELA-XXX/
├── README.md              # Resumen de la Historia
├── TASK-001.md            # Doc de Subtask 1
├── TASK-002.md            # Doc de Subtask 2
└── ...

docs/architecture/
├── ADR-XXX-titulo.md      # Decisión arquitectónica

src/
├── feature-name.py        # Código fuente

tests/unit/
├── test_feature-name.py   # Tests unitarios
```

### 5. TEMPLATE DE ADR (OBLIGATORIO)

```markdown
# ADR-XXX: [Título de la Decisión]

## Estado
✅ Aceptado | 🔄 Propuesto | ❌ Rechazado | ⏸️ Obsoleto

## Fecha
YYYY-MM-DD

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
1. **Alternativa 1**: [Descripción] - Rechazada porque [razón]
2. **Alternativa 2**: [Descripción] - Rechazada porque [razón]

## Referencias
- Jira: [VELA-XXX]
- Documentación: [link]

## Implementación
Ver código en: `src/nombre-archivo.py`
```

### 6. TEMPLATE DE CÓDIGO FUENTE

```python
"""
[Título de la Subtask]

Implementación de: VELA-XXX
Historia: VELA-YYY
Fecha: YYYY-MM-DD

Descripción:
[Descripción de lo que hace este código]
"""

class NombreClase:
    """
    Clase principal para [funcionalidad].
    
    Esta implementación resuelve [problema].
    """
    
    def __init__(self):
        """Inicializar la clase."""
        pass
    
    def metodo_principal(self):
        """
        Método principal de la funcionalidad.
        
        Returns:
            dict: Resultado de la ejecución
        """
        return {"success": True}


if __name__ == "__main__":
    instance = NombreClase()
    result = instance.metodo_principal()
    print(f"Resultado: {result}")
```

### 7. TEMPLATE DE TESTS

```python
"""
Tests unitarios para [nombre de la feature]

Jira: VELA-XXX
Historia: VELA-YYY
"""

import pytest
from src.nombre_archivo import NombreClase


class TestNombreClase:
    """Suite de tests para NombreClase."""
    
    def setup_method(self):
        """Configurar cada test."""
        self.instance = NombreClase()
    
    def test_initialization(self):
        """Test de inicialización."""
        assert self.instance is not None
    
    def test_metodo_principal(self):
        """Test del método principal."""
        result = self.instance.metodo_principal()
        assert result["success"] == True
    
    def test_metodo_principal_returns_dict(self):
        """Test que verifica el tipo de retorno."""
        result = self.instance.metodo_principal()
        assert isinstance(result, dict)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
```

### 8. TEMPLATE DE DOCUMENTACIÓN DE SUBTASK

```markdown
# TASK-XXX: [Título]

## 📋 Información General
- **Historia:** VELA-YYY
- **Estado:** Completada ✅
- **Fecha:** YYYY-MM-DD

## 🎯 Objetivo
[Qué problema resuelve esta Subtask]

## 🔨 Implementación
[Cómo se resolvió]

### Archivos generados
- `src/archivo.py` - Implementación principal
- `tests/unit/test_archivo.py` - Tests unitarios
- `docs/architecture/ADR-XXX.md` - Decisión arquitectónica (si aplica)

## ✅ Criterios de Aceptación
- [x] Código implementado
- [x] Tests escritos y pasando
- [x] Documentación generada
- [x] ADR creado (si aplica)

## 🔗 Referencias
- **Jira:** [TASK-XXX](https://velalang.atlassian.net/browse/VELA-XXX)
- **Historia:** [VELA-YYY](https://velalang.atlassian.net/browse/VELA-YYY)
```

### 9. TEMPLATE DE README DE HISTORIA

```markdown
# VELA-XXX: [Título de la Historia]

## 📋 Información General
- **Epic:** VELA-ZZZ
- **Sprint:** Sprint N
- **Estado:** Completada ✅
- **Fecha:** YYYY-MM-DD

## 🎯 Descripción
[Descripción de la Historia de Usuario]

## 📦 Subtasks Completadas
1. **TASK-XXX**: [Título] ✅
2. **TASK-YYY**: [Título] ✅

## 🔨 Implementación
Ver archivos en:
- `src/` - Código fuente
- `tests/unit/` - Tests
- `docs/features/VELA-XXX/` - Documentación

## 📊 Métricas
- **Subtasks:** X completadas
- **Archivos creados:** Y
- **Tests escritos:** Z

## ✅ Definición de Hecho
- [x] Todas las Subtasks completadas
- [x] Código funcional
- [x] Tests pasando (>= 80% cobertura)
- [x] Documentación completa
- [x] Pull Request merged

## 🔗 Referencias
- **Jira:** [VELA-XXX](https://velalang.atlassian.net/browse/VELA-XXX)
```

### 10. FORMATO DE COMMITS

```bash
# Formato OBLIGATORIO
tipo(scope): descripción breve

- Detalle 1
- Detalle 2

Refs: VELA-XXX
```

**Tipos válidos:**
- `feat`: Nueva funcionalidad
- `fix`: Corrección de bug
- `docs`: Solo documentación
- `refactor`: Refactorización
- `test`: Agregar tests
- `chore`: Tareas de mantenimiento

**Ejemplo:**
```bash
git commit -m "feat(VELA-561): implementar TASK-001 decisión de lenguaje

- ADR creado en docs/architecture/
- Código de ejemplo en src/
- Tests unitarios con 95% cobertura
- Documentación completa

Refs: VELA-561"
```

---

## 🚀 FLUJO DE TRABAJO EXACTO

### CUANDO EL USUARIO DICE: "Inicia Sprint X" o "Desarrolla Historia VELA-XXX"

**DEBES HACER:**

```python
# PASO 1: PREPARACIÓN
1. Leer la Historia en Jira
2. Obtener lista de Subtasks
3. Crear rama: git checkout -b feature/VELA-XXX-descripcion
4. Crear carpeta: docs/features/VELA-XXX/

# PASO 2: POR CADA SUBTASK (en orden)
for subtask in historia.subtasks:
    # 2.1 Mover a "En curso"
    jira.transition(subtask, "En curso")
    
    # 2.2 Identificar tipo de Subtask
    if "decidir" or "elegir" or "seleccionar" or "definir" in subtask.title:
        # Es decisión arquitectónica → GENERAR ADR
        crear_archivo(f"docs/architecture/ADR-{subtask.id}-{titulo}.md", contenido_adr)
    
    # 2.3 SIEMPRE generar código
    crear_archivo(f"src/{nombre_feature}.py", codigo_fuente)
    
    # 2.4 SIEMPRE generar tests
    crear_archivo(f"tests/unit/test_{nombre_feature}.py", tests_unitarios)
    
    # 2.5 SIEMPRE generar documentación
    crear_archivo(f"docs/features/VELA-XXX/TASK-{id}.md", documentacion)
    
    # 2.6 Commit
    git add .
    git commit -m "feat(VELA-XXX): implementar TASK-{id}
    
    - Código en src/
    - Tests en tests/unit/
    - Documentación en docs/
    - ADR en docs/architecture/ (si aplica)
    
    Refs: VELA-XXX"
    
    # 2.7 Mover a "Finalizada"
    jira.transition(subtask, "Finalizada")

# PASO 3: COMPLETAR HISTORIA
1. Generar README de Historia: docs/features/VELA-XXX/README.md
2. git add .
3. git commit -m "feat(VELA-XXX): completar historia con todos los entregables"
4. git push origin feature/VELA-XXX
5. Crear Pull Request (usar template de .github/PULL_REQUEST_TEMPLATE.md)
6. jira.transition(historia, "En revisión")
7. Mostrar al usuario: "Historia completada. PR creada. Esperando code review."

# PASO 4: DESPUÉS DE APROBACIÓN (usuario dice "aprobado" o "merge")
1. git checkout main
2. git merge feature/VELA-XXX
3. git push origin main
4. jira.transition(historia, "Finalizada")
5. git branch -d feature/VELA-XXX

# PASO 5: CIERRE DE SPRINT (cuando TODAS las Historias están listas)
1. Generar Release Notes: docs/releases/sprint-N.md
2. Actualizar CHANGELOG.md
3. git tag sprint-N
4. git push --tags
5. jira.close_sprint(sprint_id)
```

---

## ❌ ERRORES QUE NUNCA DEBES COMETER

1. ❌ **NUNCA** solo cambiar estados en Jira sin generar archivos
2. ❌ **NUNCA** crear código sin tests
3. ❌ **NUNCA** crear código sin documentación
4. ❌ **NUNCA** olvidar los ADRs en decisiones arquitectónicas
5. ❌ **NUNCA** hacer commit sin mensaje descriptivo
6. ❌ **NUNCA** crear archivos nuevos cuando hay que corregir existentes
7. ❌ **NUNCA** decir "voy a crear X" sin realmente crearlo
8. ❌ **NUNCA** asumir que las carpetas existen, SIEMPRE verificar con list_dir
9. ❌ **NUNCA JAMÁS** crear archivos con sufijos _v1, _v2, _v3, _new, _fixed, etc.
10. ❌ **NUNCA JAMÁS** crear un archivo nuevo para "arreglar" uno existente
11. ❌ **SI UN ARCHIVO TIENE ERRORES** → Usa `replace_string_in_file` para corregirlo
12. ❌ **SI TE PIDEN CORREGIR UN ARCHIVO** → Edita EL MISMO archivo, NO crees otro

---

## ✅ CHECKLIST ANTES DE MARCAR SUBTASK COMO "FINALIZADA"

```
[ ] ✅ Código creado en src/
[ ] ✅ Tests creados en tests/unit/
[ ] ✅ Tests pasando (ejecutar con pytest)
[ ] ✅ Documentación creada en docs/features/
[ ] ✅ ADR creado (si es decisión arquitectónica)
[ ] ✅ Commit realizado con mensaje descriptivo
[ ] ✅ Archivos verificados con list_dir
```

---

## 📁 ESTRUCTURA DE ARCHIVOS QUE DEBES MANTENER

```
vela/
├── .github/
│   ├── CONTRIBUTING.md           # Guía de desarrollo
│   ├── COPILOT_INSTRUCTIONS.md   # Este archivo (TUS INSTRUCCIONES)
│   ├── PULL_REQUEST_TEMPLATE.md  # Template de PR
│   └── workflows/
│       └── desarrollo-workflow.yml
│
├── docs/
│   ├── architecture/              # ADRs aquí
│   ├── features/                  # Docs por Historia
│   │   └── VELA-XXX/
│   │       ├── README.md
│   │       ├── TASK-001.md
│   │       └── TASK-002.md
│   ├── api/                       # Specs de API
│   └── design/                    # Diseños
│
├── src/                           # Código fuente aquí
├── tests/
│   ├── unit/                      # Tests unitarios aquí
│   └── integration/               # Tests integración aquí
│
├── README.md
├── CHANGELOG.md
└── .gitignore
```

---

## 🎯 COMANDOS QUE DEBES USAR

### Verificar estructura:
```bash
list_dir("C:\\Users\\cristian.naranjo\\Downloads\\Vela")
list_dir("C:\\Users\\cristian.naranjo\\Downloads\\Vela\\src")
list_dir("C:\\Users\\cristian.naranjo\\Downloads\\Vela\\docs\\features")
```

### Crear archivos:
```bash
create_file(path, content)  # Solo si el archivo NO existe
replace_string_in_file(path, old, new)  # Si el archivo SÍ existe (para corregir)
```

### Git:
```bash
run_in_terminal("git status")
run_in_terminal("git add .")
run_in_terminal("git commit -m 'mensaje'")
run_in_terminal("git push")
```

---

## 🔄 INTERACCIÓN CON JIRA

### Estados válidos (en ESPAÑOL):
- **"Tareas por hacer"** - Estado inicial
- **"En curso"** - Trabajo en progreso
- **"En revisión"** - Esperando code review
- **"Finalizada"** - Completado

### Transiciones:
```python
# Iniciar Subtask
jira.transition(subtask_key, "En curso")

# Completar Subtask
jira.transition(subtask_key, "Finalizada")

# Mover Historia a revisión
jira.transition(historia_key, "En revisión")

# Completar Historia
jira.transition(historia_key, "Finalizada")

# Cerrar Sprint
jira.close_sprint(sprint_id)
```

---

## 📊 MÉTRICAS QUE DEBES REPORTAR

Al finalizar cada Historia:
```
✅ HISTORIA COMPLETADA: VELA-XXX

📊 Métricas:
- Subtasks completadas: X
- Archivos creados: Y
  - ADRs: Z
  - Código fuente: A
  - Tests: B
  - Documentación: C
- Commits realizados: D
- Tests pasando: E/E (100%)

📁 Ubicación de archivos:
- docs/features/VELA-XXX/
- src/
- tests/unit/
```

---

## 🚨 SI ENCUENTRAS ERRORES EN UN ARCHIVO

### ⚠️ REGLA DE ORO: NUNCA CREAR ARCHIVOS NUEVOS PARA CORREGIR

Si `archivo.py` tiene errores:
- ✅ **CORRECTO**: `replace_string_in_file("archivo.py", codigo_malo, codigo_bueno)`
- ❌ **INCORRECTO**: `create_file("archivo_v2.py", codigo_bueno)`
- ❌ **INCORRECTO**: `create_file("archivo_fixed.py", codigo_bueno)`
- ❌ **INCORRECTO**: `create_file("archivo_new.py", codigo_bueno)`

### Proceso de corrección:

1. **Leer el error completo**
2. **Identificar el archivo con error**
3. **Leer el contenido del archivo** con `read_file`
4. **Usar `replace_string_in_file`** para corregir
5. **Verificar que la corrección funcione**
6. **NUNCA, BAJO NINGUNA CIRCUNSTANCIA, crear archivo_v2, archivo_v3, etc.**

### Ejemplo de corrección CORRECTA:

```python
# ❌ MAL
create_file("develop_historia_v2.py", codigo_corregido)

# ✅ BIEN
replace_string_in_file(
    filePath="develop_historia.py",
    oldString="codigo con error",
    newString="codigo corregido"
)
```

### Si el usuario dice "arregla este archivo":
1. Abrir el archivo con `read_file`
2. Identificar el problema
3. Usar `replace_string_in_file` para corregir
4. **NO crear develop_historia_v2.py, develop_historia_fixed.py, etc.**

---

## 💡 EJEMPLO COMPLETO DE DESARROLLO

```
USUARIO: "Desarrolla la Historia VELA-561"

TÚ DEBES:

1. list_dir para verificar estructura
2. git checkout -b feature/VELA-561
3. Para TASK-001 (Decidir lenguaje):
   - jira.transition(TASK-001, "En curso")
   - create_file("docs/architecture/ADR-001-decidir-lenguaje.md", adr_content)
   - create_file("src/language_decision.py", code_content)
   - create_file("tests/unit/test_language_decision.py", test_content)
   - create_file("docs/features/VELA-561/TASK-001.md", doc_content)
   - git commit -m "feat(VELA-561): implementar TASK-001..."
   - jira.transition(TASK-001, "Finalizada")

4. Repetir para TASK-002, TASK-003, etc.

5. create_file("docs/features/VELA-561/README.md", readme_content)

6. git push y crear PR

7. Reportar al usuario:
   "✅ Historia VELA-561 completada
   📦 12 archivos generados
   📁 Ver: docs/features/VELA-561/"
```

---

## 📞 PREGUNTAS AL USUARIO

Si algo no está claro:
- ❓ "¿Qué Historia debo desarrollar?"
- ❓ "¿Apruebo el Pull Request y hago merge?"
- ❓ "¿Cierro el Sprint?"

**NUNCA** asumas respuestas.

---

**ÚLTIMA ACTUALIZACIÓN:** 2025-11-30  
**VERSIÓN:** 1.0.0

**RECUERDA: Este archivo contiene TUS INSTRUCCIONES. Léelo SIEMPRE antes de desarrollar una Historia.**
