# ⚠️ ESTE ARCHIVO HA SIDO REEMPLAZADO

## � Nueva Ubicación de la Documentación

Este documento ha sido **transformado y expandido** en una guía completa de GitHub.

### 🔗 LEE LA NUEVA GUÍA AQUÍ:

**`../.github/CONTRIBUTING.md`**

---

## 🎯 ¿Qué cambió?

### **ANTES (este archivo):**
- ❌ Solo cambios de estado en Jira
- ❌ Sin entregables tangibles
- ❌ Sin código real generado

### **AHORA (.github/CONTRIBUTING.md):**
- ✅ **Cada tarea genera entregables REALES**
- ✅ Código fuente funcional
- ✅ Tests unitarios (>= 80% cobertura)
- ✅ Documentación completa
- ✅ ADRs (Architecture Decision Records)
- ✅ Control de versiones con Git
- ✅ Pull Requests estructurados

---

## 📚 Archivos Importantes del Nuevo Sistema

### 1️⃣ **`.github/CONTRIBUTING.md`** ⭐⭐⭐
**Guía COMPLETA de desarrollo**
- Principios fundamentales
- Estructura del proyecto
- Flujo de trabajo (3 fases)
- **Entregables por tipo de tarea** (tabla completa)
- Templates de ADR
- Estándares de calidad
- Checklists de revisión

### 2️⃣ **`develop_historia_v2.py`** ⭐⭐⭐
**Script de automatización MEJORADO**
- Genera ADRs (Architecture Decision Records)
- Genera código fuente con docstrings
- Genera tests unitarios con pytest
- Genera documentación por Subtask
- Genera README por Historia
- Actualiza estados en Jira
- Crea estructura de Git

### 3️⃣ **`README.md`**
Documentación principal del proyecto

### 4️⃣ **`CHANGELOG.md`**
Historial de cambios por Sprint

### 5️⃣ **`RESUMEN_DE_CAMBIOS.md`**
Resumen completo de la transformación

---

## 🎯 Para GitHub Copilot

### **SIEMPRE LEE:**
- **`.github/CONTRIBUTING.md`** - Es la fuente de verdad

### **EJECUTA:**
```bash
cd C:\Users\cristian.naranjo\Downloads\Vela\jira-import
python develop_historia_v2.py
```

### **RECUERDA:**
1. ✅ Cada tarea DEBE generar entregables tangibles
2. ✅ Todo código DEBE tener tests
3. ✅ Toda decisión arquitectónica DEBE tener ADR
4. ✅ Todo cambio DEBE estar en Git
5. ✅ Todo desarrollo DEBE pasar code review

---

**Fecha de cambio:** 2025-11-30  
**Motivo:** Transformar proceso de solo cambios de estado a generación de entregables reales

---

# �📋 INSTRUCCIONES PERMANENTES - PROCESO DE DESARROLLO SCRUM (OBSOLETO)

## 🎯 OBJETIVO
Automatizar completamente el ciclo de desarrollo de Historias de Usuario en Jira, desde el inicio hasta el cierre del Sprint.

---

## 🔄 PROCESO COMPLETO DE DESARROLLO

### **FASE 1: Iniciar Sprint**
El usuario inicia el Sprint manualmente desde Jira.

**Tu acción:** Confirmar qué Sprint está activo.

---

### **FASE 2: Desarrollar Historia de Usuario**

#### **Paso 1: Identificar Historia**
- Obtener la Historia activa del Sprint actual
- Historia = US-XXX (ejemplo: US-00A)
- Key de Jira = VELA-XXX

#### **Paso 2: Cambiar estado de Historia a "In Progress"**
```
Historia: To Do → In Progress
```

#### **Paso 3: Desarrollar cada Subtask**
Para CADA Subtask de la Historia:

1. **Iniciar Subtask**
   ```
   Subtask: To Do → In Progress
   ```

2. **Simular desarrollo** (espera 2-3 segundos)
   - Este tiempo representa el desarrollo real
   - En producción, aquí irían los cambios de código reales

3. **Completar Subtask**
   ```
   Subtask: In Progress → Done
   ```

4. **Repetir** para todos los Subtasks de la Historia

#### **Paso 4: Completar Historia**
Cuando TODOS los Subtasks estén en "Done":
```
Historia: In Progress → Done
```

---

### **FASE 3: Cerrar Sprint**

Cuando TODAS las Historias del Sprint estén en "Done":

1. **Verificar estado del Sprint**
   - Confirmar que todas las Historias están completas
   - Verificar que no hay issues pendientes

2. **Cerrar Sprint**
   ```
   Sprint: Active → Closed
   ```

---

## 🛠️ SCRIPTS DISPONIBLES

### **develop_historia.py**
**Propósito:** Desarrollar una Historia completa con todos sus Subtasks

**Uso:**
```bash
python develop_historia.py
```

**Proceso automático:**
1. Mueve Historia a "In Progress"
2. Para cada Subtask:
   - Mueve a "In Progress"
   - Simula desarrollo (2 seg)
   - Mueve a "Done"
3. Mueve Historia a "Done"
4. Opción de cerrar Sprint

**Parámetros configurables (en el script):**
- `HISTORIA_KEY`: Key de Jira de la Historia (ej: VELA-560)
- `SPRINT_ID`: ID del Sprint a cerrar (ej: 174)

---

## 📝 FLUJO DE TRABAJO COMPLETO

### **Para cada Sprint:**

```
1. Usuario: Inicia Sprint N desde Jira
   ↓
2. Copilot: Ejecuta develop_historia.py para cada Historia del Sprint
   - Historia 1 (US-XXX): To Do → In Progress → Done
   - Historia 2 (US-YYY): To Do → In Progress → Done
   - ...
   ↓
3. Copilot: Cierra Sprint N
   ↓
4. Usuario: Inicia Sprint N+1
   ↓
5. REPETIR desde paso 2
```

---

## 🔧 CONFIGURACIÓN NECESARIA

### **Variables en config.py:**
- `JIRA_URL`: URL de la instancia Jira
- `JIRA_EMAIL`: Email de autenticación
- `JIRA_API_TOKEN`: Token de API
- `PROJECT_KEY`: Clave del proyecto (VELA)

### **IDs de Sprints:**
Los IDs se obtienen al crear los sprints:
- Sprint 0: 174
- Sprint 1: 175
- Sprint 2: 176
- ...
- Sprint 64: 238

---

## 📊 ESTADOS DE JIRA

### **Estados en español (tu instancia):**
- **Tareas por hacer**: Estado inicial
- **En curso**: Trabajo en progreso
- **En revisión**: En revisión (opcional)
- **Finalizada**: Trabajo completado

### **Transiciones válidas:**
1. **Tareas por hacer → En curso**: Iniciar trabajo
2. **En curso → Finalizada**: Completar trabajo
3. **Sprint Active → Closed**: Cerrar sprint

### **Jerarquía de issues:**
```
Epic (sin sprint)
  └─ Historia (con sprint)
       └─ Subtask (hereda sprint del padre)
```

---

## ⚠️ IMPORTANTE - REGLAS DE ORO

1. **NUNCA saltar estados**: Siempre pasar por In Progress antes de Done
2. **NUNCA cerrar Historia antes que sus Subtasks**: Todos los Subtasks deben estar Done primero
3. **NUNCA cerrar Sprint con issues pendientes**: Todas las Historias deben estar Done
4. **SIEMPRE esperar entre transiciones**: time.sleep(1-2) para evitar race conditions
5. **SIEMPRE verificar transiciones disponibles**: Usar get_transitions() antes de mover

---

## 🚀 EJEMPLO DE EJECUCIÓN

### **Sprint 0 - Historia US-00A (VELA-560)**

```bash
cd C:\Users\cristian.naranjo\Downloads\Vela\jira-import
python develop_historia.py
```

**Salida esperada:**
```
🚀 DESARROLLANDO HISTORIA: VELA-560
📋 Historia: US-00A: Como líder técnico, necesito...
📊 Subtasks: 5

🔄 Iniciando Historia VELA-560...
✅ Historia estado: In Progress

💻 DESARROLLANDO SUBTASKS

[1/5]
📌 VELA-1195: TASK-000A: Decidir lenguaje de implementación...
   🔄 Iniciando desarrollo...
   ✅ Estado: In Progress
   💻 Desarrollando...
   🔄 Completando...
   ✅ Estado: Done

[2/5]
📌 VELA-1196: TASK-000B: Definir arquitectura del build system...
   ...

[Continúa con todos los Subtasks]

🔄 Completando Historia VELA-560...
✅ Historia estado: Done

✅ HISTORIA COMPLETADA: VELA-560

¿Cerrar Sprint 0 ahora? (SI/NO): SI

🏁 CERRANDO SPRINT 174
✅ Sprint 174 cerrado exitosamente

✅ PROCESO COMPLETADO
```

---

## 📋 CHECKLIST PARA COPILOT

Antes de ejecutar el desarrollo:

- [ ] Confirmar que el Sprint está iniciado (estado: Active)
- [ ] Identificar la Historia a desarrollar (US-XXX, VELA-XXX)
- [ ] Verificar que la Historia tiene Subtasks
- [ ] Confirmar SPRINT_ID correcto en el script
- [ ] Ejecutar develop_historia.py
- [ ] Verificar que todos los Subtasks quedaron en Done
- [ ] Verificar que la Historia quedó en Done
- [ ] Si es la última Historia del Sprint, cerrar Sprint

---

## 🔄 PRÓXIMOS PASOS DESPUÉS DE CERRAR SPRINT

1. Usuario inicia Sprint siguiente
2. Actualizar HISTORIA_KEY y SPRINT_ID en develop_historia.py
3. Ejecutar develop_historia.py para la primera Historia del nuevo Sprint
4. Repetir proceso

---

## 💾 ARCHIVO DE REFERENCIA

**Ubicación:** `C:\Users\cristian.naranjo\Downloads\Vela\jira-import\DESARROLLO_WORKFLOW.md`

**Mantenlo siempre actualizado** con:
- Nuevos scripts creados
- Cambios en el flujo de trabajo
- IDs de Sprints completados
- Problemas encontrados y soluciones

---

## 🎯 RECORDATORIO CLAVE

**Siempre que el usuario diga:**
- "Desarrolla la siguiente Historia"
- "Continúa con el Sprint"
- "Completa US-XXX"

**Tu acción inmediata:**
1. Identificar HISTORIA_KEY y SPRINT_ID
2. Actualizar develop_historia.py si es necesario
3. Ejecutar: `python develop_historia.py`
4. Si es la última Historia, preguntar si cerrar Sprint
5. Si cierra Sprint, prepararte para el siguiente

---

**FIN DE INSTRUCCIONES**
