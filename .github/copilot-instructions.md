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

## 🔥 ESPECIFICACIÓN COMPLETA DEL LENGUAJE VELA

### ⚠️ PARADIGMA: PROGRAMACIÓN FUNCIONAL PURA

**Vela es un lenguaje FUNCIONAL PURO con reactividad y UI declarativa.**

---

### ❌ PALABRAS RESERVADAS QUE **NO EXISTEN** EN VELA

**NUNCA USES ESTAS KEYWORDS (NO ESTÁN EN EL LENGUAJE):**

#### Loops Imperativos (PROHIBIDOS):
- ❌ `for` - NO EXISTE (usar métodos funcionales: `.map()`, `.forEach()`, `.filter()`)
- ❌ `while` - NO EXISTE (usar recursión o métodos funcionales)
- ❌ `loop` - NO EXISTE (usar recursión tail-call optimizada)
- ❌ `break` - NO EXISTE (no hay loops)
- ❌ `continue` - NO EXISTE (no hay loops)
- ❌ `do` - NO EXISTE (no hay do-while)

#### Mutabilidad por Defecto (PROHIBIDO):
- ❌ `let` - NO EXISTE (variables son inmutables por defecto)
- ❌ `const` - NO EXISTE (inmutabilidad es por defecto, NO necesita keyword)
- ❌ `var` - NO EXISTE (jamás)
- ❌ `mut` - NO EXISTE (usar `state` para mutabilidad reactiva)

#### Valores Especiales (PROHIBIDOS):
- ❌ `null` - NO EXISTE (usar `None` en `Option<T>`)
- ❌ `undefined` - NO EXISTE (usar `Option<T>`)
- ❌ `nil` - NO EXISTE (usar `None`)

#### Exports Explícitos (PROHIBIDO):
- ❌ `export` - NO EXISTE (usar modificador `public` en lugar)
- ❌ `module` - NO EXISTE (usar estructura de carpetas)

#### Otros (PROHIBIDOS):
- ❌ `switch` - NO EXISTE (usar `match` con pattern matching)
- ❌ `case` - NO EXISTE (usar `match`)
- ❌ `default` - NO EXISTE (usar `_` en match)
- ❌ `goto` - NO EXISTE (jamás)
- ❌ `with` - NO EXISTE
- ❌ `in` - NO EXISTE como keyword standalone

---

### ✅ PALABRAS RESERVADAS QUE **SÍ EXISTEN** EN VELA

#### 1. Declaración de Variables

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `state` | Variable **mutable y reactiva** (ÚNICA forma de mutabilidad) | `state count: Number = 0` |
| *(sin keyword)* | Inmutable por defecto (NO necesita `const` ni `let`) | `name: String = "Vela"` |

**Regla de Oro**: 
- ✅ Variables sin keyword → **Inmutables** (99% de los casos)
- ✅ `state` → **Mutable y reactiva** (solo para estado UI)

---

#### 2. Tipos de Datos Primitivos

| Keyword | Descripción | Ejemplo |
|---------|-------------|---------|
| `Number` | Entero (64-bit) | `age: Number = 37` |
| `Float` | Punto flotante (64-bit) | `price: Float = 19.99` |
| `String` | Cadena de texto | `name: String = "Vela"` |
| `Bool` | Booleano | `isActive: Bool = true` |
| `void` | Sin retorno | `fn log() -> void { }` |
| `never` | Nunca retorna (throw o loop infinito) | `fn panic() -> never { throw Error() }` |

**Valores Especiales**:
- ✅ `true` / `false` (booleanos)
- ✅ `None` (en lugar de null/undefined/nil)
- ✅ `Option<T>` (manejo de valores opcionales: `Some(value)` o `None`)

---

#### 3. Estructuras de Datos

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `type` | Alias de tipo o union type | `type UserId = Number` o `type Status = "active" \| "inactive"` |
| `enum` | Enumeración (con o sin datos asociados) | `enum Color { Red, Green, Blue, Custom(r, g, b) }` |
| `struct` | Estructura de datos (record/producto) | `struct User { id: Number, name: String }` |

---

#### 4. POO (Programación Orientada a Objetos)

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `class` | Define una clase | `class Person { ... }` |
| `abstract` | Clase abstracta (no instanciable) | `abstract class Shape { abstract fn area() -> Float }` |
| `interface` | Contrato de tipo | `interface Drawable { fn draw() -> void }` |
| `extends` | Herencia | `class Dog extends Animal { }` |
| `implements` | Implementa interfaz | `class Button implements Clickable { }` |
| `override` | Sobrescribe método padre | `override fn toString() -> String { }` |
| `overload` | Sobrecarga de métodos | `overload fn add(a: Number, b: Number) -> Number { }` |
| `this` | Instancia actual | `this.name` |
| `super` | Clase padre | `super.greet()` |
| `constructor` | Constructor de clase | `constructor(name: String) { this.name = name }` |

---

#### 5. Funciones

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `fn` | Define función | `fn add(a: Number, b: Number) -> Number { return a + b }` |
| `async` | Función asíncrona | `async fn fetchData() -> Result<String> { }` |
| `await` | Espera resultado async | `data = await fetchData()` |
| `return` | Retorna valor | `return result` |
| `yield` | Generador (produce valor) | `yield nextValue` |

**Arrow Functions**:
```vela
# ✅ Función anónima
callback = (x: Number) => x * 2

# ✅ Con bloque
process = (data: String) => {
  cleaned = data.trim()
  return cleaned.toUpperCase()
}
```

---

#### 6. Control de Flujo (FUNCIONAL)

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `if` | Condicional (también expression) | `if age >= 18 { "adult" } else { "minor" }` |
| `else` | Rama alternativa | `if x > 0 { ... } else { ... }` |
| `match` | Pattern matching (exhaustivo) | `match result { Ok(val) => ..., Err(e) => ... }` |

**⚠️ NO HAY LOOPS IMPERATIVOS**:
```vela
# ❌ PROHIBIDO: for loop
# for i in 0..10 { print(i) }

# ✅ CORRECTO: métodos funcionales
(0..10).forEach(i => print(i))

# ❌ PROHIBIDO: while loop
# while condition { doSomething() }

# ✅ CORRECTO: recursión
fn repeatUntil(condition: () -> Bool, action: () -> void) -> void {
  if !condition() {
    action()
    repeatUntil(condition, action)  # tail-call optimizado
  }
}

# ❌ PROHIBIDO: loop infinito
# loop { process() }

# ✅ CORRECTO: recursión infinita (tail-call)
fn processForever() -> never {
  process()
  processForever()
}
```

---

#### 7. Métodos Funcionales de Listas (OBLIGATORIO USAR)

**En lugar de loops, usar estos métodos funcionales:**

| Método | Propósito | Ejemplo |
|--------|-----------|---------|
| `.map()` | Transformar elementos | `[1, 2, 3].map(x => x * 2)` → `[2, 4, 6]` |
| `.filter()` | Filtrar elementos | `[1, 2, 3, 4].filter(x => x % 2 == 0)` → `[2, 4]` |
| `.reduce()` | Reducir a un valor | `[1, 2, 3].reduce((acc, x) => acc + x, 0)` → `6` |
| `.forEach()` | Ejecutar acción por elemento | `list.forEach(x => print(x))` |
| `.flatMap()` | Mapear y aplanar | `[[1, 2], [3]].flatMap(x => x)` → `[1, 2, 3]` |
| `.find()` | Encontrar primer match | `list.find(x => x > 5)` → `Some(6)` o `None` |
| `.findIndex()` | Índice del primer match | `list.findIndex(x => x > 5)` → `Some(3)` o `None` |
| `.every()` | Todos cumplen condición | `[2, 4, 6].every(x => x % 2 == 0)` → `true` |
| `.some()` | Al menos uno cumple | `[1, 2, 3].some(x => x % 2 == 0)` → `true` |
| `.take()` | Primeros N elementos | `[1, 2, 3, 4].take(2)` → `[1, 2]` |
| `.drop()` | Saltar primeros N | `[1, 2, 3, 4].drop(2)` → `[3, 4]` |
| `.takeWhile()` | Tomar mientras condición | `[1, 2, 3, 4].takeWhile(x => x < 3)` → `[1, 2]` |
| `.dropWhile()` | Saltar mientras condición | `[1, 2, 3, 4].dropWhile(x => x < 3)` → `[3, 4]` |
| `.partition()` | Dividir en dos listas | `[1, 2, 3, 4].partition(x => x % 2 == 0)` → `([2, 4], [1, 3])` |
| `.groupBy()` | Agrupar por clave | `["a", "ab", "abc"].groupBy(s => s.length)` |
| `.sortBy()` | Ordenar por criterio | `list.sortBy(x => x.age)` |
| `.chunk()` | Dividir en grupos | `[1, 2, 3, 4, 5].chunk(2)` → `[[1, 2], [3, 4], [5]]` |
| `.zip()` | Combinar dos listas | `[1, 2].zip(["a", "b"])` → `[(1, "a"), (2, "b")]` |
| `.scan()` | Reduce con pasos intermedios | `[1, 2, 3].scan((a, b) => a + b, 0)` → `[0, 1, 3, 6]` |
| `.distinct()` | Eliminar duplicados | `[1, 2, 2, 3].distinct()` → `[1, 2, 3]` |
| `.reverse()` | Invertir orden | `[1, 2, 3].reverse()` → `[3, 2, 1]` |

---

#### 8. Manejo de Errores

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `try` | Bloque try-catch | `try { riskyOp() } catch (e) { handle(e) }` |
| `catch` | Captura excepción | `catch (e: MyError) { ... }` |
| `throw` | Lanza excepción | `throw Error("failed")` |
| `finally` | Siempre se ejecuta | `finally { cleanup() }` |

**Tipo `Result<T, E>`** (preferido sobre excepciones):
```vela
fn divide(a: Number, b: Number) -> Result<Float, Error> {
  if b == 0 {
    return Err(Error("division by zero"))
  }
  return Ok(a / b)
}

# Uso con match
match divide(10, 2) {
  Ok(value) => print("Result: ${value}")
  Err(error) => print("Error: ${error}")
}
```

---

#### 9. Imports y Módulos

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `import` | Importar módulo/paquete | `import 'package:http'` |
| `show` | Importar elementos específicos | `import 'lib:utils' show { sort, filter }` |
| `hide` | Importar todo excepto | `import 'lib:math' hide { deprecated_fn }` |
| `as` | Alias para import | `import 'package:long_name' as ln` |

**⚠️ NO EXISTE `export`**:
```vela
# ❌ PROHIBIDO: export keyword
# export fn myFunction() { }

# ✅ CORRECTO: modificador public
public fn myFunction() -> void {
  # accesible desde otros módulos
}

# Privado por defecto (sin modificador)
fn privateHelper() -> void {
  # solo accesible dentro del módulo
}
```

---

#### 10. Modificadores de Acceso

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `public` | Accesible públicamente | `public class MyClass { }` |
| `private` | Solo dentro de clase/módulo | `private fn helper() -> void { }` |
| `protected` | Clase y subclases | `protected fn method() -> void { }` |

---

#### 11. Reactividad (Sistema Reactivo Integrado)

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `computed` | Valor derivado reactivo | `computed doubled: Number { return this.count * 2 }` |
| `memo` | Computed con caché agresivo | `memo expensive: Number { /* cálculo costoso */ }` |
| `effect` | Side effect reactivo | `effect { print("Count: ${this.count}") }` |
| `watch` | Observar cambios específicos | `watch(this.name) { print("Name changed") }` |

---

#### 12. Ciclo de Vida de Componentes UI

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `mount` | Hook al montar componente | `mount() { this.fetchData() }` |
| `update` | Hook después de actualización | `update() { print("Updated") }` |
| `destroy` | Hook al desmontar | `destroy() { this.cleanup() }` |
| `beforeUpdate` | Antes de actualizar DOM | `beforeUpdate() { /* ... */ }` |
| `afterUpdate` | Después de actualizar DOM | `afterUpdate() { /* ... */ }` |

---

#### 13. UI - Widgets (Inspirado en Flutter)

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `StatefulWidget` | Widget con estado mutable | `class Counter extends StatefulWidget { state count: Number = 0 }` |
| `StatelessWidget` | Widget sin estado (puro) | `class Label extends StatelessWidget { text: String }` |
| `component` | Componente UI (alias de StatefulWidget) | `component MyButton { /* ... */ }` |
| `widget` | Define un widget genérico | `widget CustomBox { /* ... */ }` |

---

#### 14. Arquitectura / Domain-Driven Design

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `service` | Capa de servicio (lógica de negocio) | `service UserService { fn createUser() { } }` |
| `repository` | Capa de acceso a datos | `repository UserRepository { fn findById() { } }` |
| `controller` | Controlador (HTTP, etc.) | `controller UserController { fn handleRequest() { } }` |
| `usecase` | Caso de uso / interactor | `usecase CreateUser { fn execute() { } }` |
| `entity` | Entidad de dominio | `entity User { id: UserId, name: String }` |
| `dto` | Data Transfer Object | `dto CreateUserDTO { name: String, email: String }` |
| `valueObject` | Value Object (inmutable) | `valueObject Email { value: String }` |
| `model` | Modelo genérico | `model Product { /* ... */ }` |

---

#### 15. Patrones de Diseño (Keywords First-Class)

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `factory` | Factory pattern | `factory UserFactory { fn create() -> User { } }` |
| `builder` | Builder pattern | `builder QueryBuilder { fn where() -> Self { } }` |
| `strategy` | Strategy pattern | `strategy PaymentStrategy { fn pay() { } }` |
| `observer` | Observer pattern | `observer EventObserver { fn notify() { } }` |
| `singleton` | Singleton pattern | `singleton Database { /* instancia única */ }` |
| `adapter` | Adapter pattern | `adapter LegacyAdapter { fn adapt() { } }` |
| `decorator` | Decorator pattern | `decorator LogDecorator { fn wrap() { } }` |

---

#### 16. Web / API (Middleware, Guards, etc.)

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `guard` | Route guard (autorización) | `guard AuthGuard { fn canActivate() -> Bool { } }` |
| `middleware` | HTTP middleware | `middleware Logger { fn handle() { } }` |
| `interceptor` | Request/response interceptor | `interceptor AuthInterceptor { fn intercept() { } }` |
| `validator` | Validador de input | `validator EmailValidator { fn validate() -> Bool { } }` |
| `pipe` | Pipeline de transformación | `pipe TransformPipe { fn transform() { } }` |

---

#### 17. Utilidades

| Keyword | Propósito | Ejemplo |
|---------|-----------|---------|
| `task` | Tarea asíncrona/job | `task EmailTask { async fn run() { } }` |
| `helper` | Helper/utilidad | `helper DateHelper { fn format() -> String { } }` |
| `mapper` | Object mapper | `mapper UserMapper { fn toDTO() -> UserDTO { } }` |
| `serializer` | Serializador de datos | `serializer JsonSerializer { fn serialize() { } }` |
| `provider` | Proveedor de dependencias | `provider ServiceProvider { fn provide() { } }` |
| `store` | Store global (estado) | `store AppStore { state count: Number = 0 }` |

---

### 🎨 DECORADORES / ANNOTATIONS

**Decoradores para DI (Dependency Injection)**:
- `@injectable` - Marca clase como inyectable
- `@inject` - Inyecta dependencia
- `@singleton` - Instancia única
- `@provides` - Proveedor de dependencia
- `@container` - Contenedor de DI

**Decoradores para HTTP**:
- `@get(path)` - HTTP GET endpoint
- `@post(path)` - HTTP POST endpoint
- `@put(path)` - HTTP PUT endpoint
- `@patch(path)` - HTTP PATCH endpoint
- `@delete(path)` - HTTP DELETE endpoint

**Decoradores para Validación**:
- `@validate` - Validar input
- `@required` - Campo requerido
- `@min(n)` - Valor mínimo
- `@max(n)` - Valor máximo
- `@email` - Validar email
- `@url` - Validar URL

**Ejemplo de uso**:
```vela
@injectable
service UserService {
  repository: UserRepository = inject(UserRepository)
  
  @validate
  fn createUser(@required name: String, @email email: String) -> Result<User> {
    # ...
  }
}

@injectable
@singleton
class DatabaseConnection {
  # solo una instancia en toda la app
}

controller UserController {
  @get("/users/:id")
  async fn getUser(id: Number) -> Result<User> {
    # ...
  }
  
  @post("/users")
  @validate
  async fn createUser(dto: CreateUserDTO) -> Result<User> {
    # ...
  }
}
```

---

### 🔄 OPCIONALIDAD: `Option<T>` en lugar de null

**Vela usa `Option<T>` para valores opcionales:**

```vela
# ✅ CORRECTO: usar Option<T>
fn findUser(id: Number) -> Option<User> {
  user = database.query(id)
  if user.exists() {
    return Some(user)
  }
  return None
}

# Usar con match (exhaustivo)
match findUser(123) {
  Some(user) => print("Found: ${user.name}")
  None => print("User not found")
}

# Usar con if-let
if let Some(user) = findUser(123) {
  print("Found: ${user.name}")
}

# Unwrap con default
user = findUser(123).unwrapOr(defaultUser)

# Chaining con map
userName = findUser(123).map(u => u.name).unwrapOr("Unknown")
```

**❌ PROHIBIDO usar `null`, `undefined`, `nil`**:
```vela
# ❌ ERROR: null no existe
# user: User? = null

# ✅ CORRECTO: usar Option<T>
user: Option<User> = None
```

---

### 📝 SINTAXIS ESPECÍFICA DE VELA

#### Interpolación de Strings
```vela
# ✅ CORRECTO: usar ${}
name: String = "Vela"
message: String = "Hello, ${name}!"
complex: String = "Result: ${calculate(x, y)}"

# ❌ PROHIBIDO: backticks o +
# message = `Hello, ${name}`  // ERROR
# message = "Hello, " + name  // Poco idiomático
```

#### Rangos
```vela
# Rango exclusivo: 0..10 → [0, 1, 2, ..., 9]
(0..10).forEach(i => print(i))

# Rango inclusivo: 0..=10 → [0, 1, 2, ..., 10]
(0..=10).forEach(i => print(i))
```

#### Pattern Matching Exhaustivo
```vela
# match DEBE cubrir todos los casos
match value {
  1 => "one"
  2 => "two"
  _ => "other"  # catch-all obligatorio
}

# Destructuring
match point {
  { x: 0, y: 0 } => "origin"
  { x, y } => "point at (${x}, ${y})"
}

# Guards
match number {
  n if n < 0 => "negative"
  n if n == 0 => "zero"
  n => "positive"
}
```

#### Inmutabilidad por Defecto
```vela
# ✅ CORRECTO: inmutable sin keyword
PI: Float = 3.14159
name: String = "Vela"

# ❌ ERROR: intentar mutar inmutable
# name = "Otro"  // ERROR de compilación

# ✅ CORRECTO: usar state para mutabilidad
state counter: Number = 0
counter = counter + 1  # OK

# ✅ CORRECTO: crear nueva variable (shadowing)
x: Number = 5
x: Number = x + 1  # Nueva variable x (shadow), NO mutación
```

---

### 🧪 TESTING

```vela
# Tests con decorador @test
@test
fn testAddition() -> void {
  result = add(2, 3)
  assert(result == 5, "2 + 3 should equal 5")
}

@test
async fn testAsyncOperation() -> void {
  result = await fetchData()
  assert(result.isOk(), "Fetch should succeed")
}

# Test con setup/teardown
@beforeEach
fn setup() -> void {
  database.connect()
}

@afterEach
fn teardown() -> void {
  database.disconnect()
}
```

---

### 📋 RESUMEN DE DECISIONES CLAVE

| Decisión | Razón | Alternativa Prohibida |
|----------|-------|----------------------|
| **Inmutabilidad por defecto** | Seguridad, funcional puro | ❌ `let`, `const`, `var` |
| **`state` para mutabilidad** | Reactividad integrada | ❌ `mut`, mutabilidad implícita |
| **`Option<T>` en lugar de null** | Seguridad de tipos, no NPE | ❌ `null`, `undefined`, `nil` |
| **Métodos funcionales en lugar de loops** | Funcional puro, composición | ❌ `for`, `while`, `loop` |
| **`match` en lugar de switch** | Pattern matching exhaustivo | ❌ `switch`, `case` |
| **`public` en lugar de export** | Consistencia con modificadores | ❌ `export` keyword |
| **`Result<T, E>` sobre excepciones** | Control explícito de errores | ⚠️ `throw` permitido pero no idiomático |
| **Decoradores para metadata** | Declarativo, menos boilerplate | ❌ Configuración manual |

---

### ✅ CHECKLIST AL ESCRIBIR CÓDIGO VELA

Antes de generar código, VERIFICA:

- [ ] ❌ NO usar `for`, `while`, `loop`, `break`, `continue`
- [ ] ✅ Usar métodos funcionales (`.map()`, `.filter()`, `.forEach()`, etc.)
- [ ] ❌ NO usar `null`, `undefined`, `nil`
- [ ] ✅ Usar `Option<T>` con `Some()` y `None`
- [ ] ❌ NO usar `let`, `const`, `var`
- [ ] ✅ Variables inmutables por defecto (sin keyword)
- [ ] ✅ Usar `state` SOLO para estado reactivo mutable
- [ ] ❌ NO usar `export` keyword
- [ ] ✅ Usar modificador `public` para exports
- [ ] ❌ NO usar `switch` / `case`
- [ ] ✅ Usar `match` con pattern matching
- [ ] ✅ Interpolación de strings con `${}`
- [ ] ✅ `Result<T, E>` para manejo de errores
- [ ] ✅ Funciones puras sin side effects (salvo `effect` explícito)
- [ ] ✅ Decoradores (`@injectable`, `@get`, `@validate`, etc.)

---

**ÚLTIMA ACTUALIZACIÓN:** 2025-11-30  
**VERSIÓN:** 2.0.0  
**CAMBIOS:** Agregada especificación completa del lenguaje Vela (paradigma funcional, palabras prohibidas, sintaxis específica)

**RECUERDA: Este archivo contiene TUS INSTRUCCIONES. Léelo SIEMPRE antes de desarrollar una Historia.**
