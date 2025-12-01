# 🔧 Scripts de Gestión de Jira - Proyecto Vela

## ⚠️ DOCUMENTACIÓN PRINCIPAL MOVIDA

**La documentación completa del proceso de desarrollo está ahora en:**

**`.github/CONTRIBUTING.md`** ⭐⭐⭐

---

## 🚀 Script Principal: `develop_historia_v2.py`

### **¿Qué hace?**

Automatiza el desarrollo de Historias de Usuario generando **entregables REALES**:

- ✅ **ADRs** (Architecture Decision Records)
- ✅ **Código fuente** funcional con docstrings
- ✅ **Tests unitarios** con pytest (>= 80% cobertura)
- ✅ **Documentación** por Subtask
- ✅ **README** por Historia de Usuario
- ✅ Actualiza estados en Jira

### **Uso:**

```bash
# 1. Editar el script
#    - HISTORIA_KEY = "VELA-XXX"
#    - SPRINT_ID = XXX

# 2. Ejecutar
python develop_historia_v2.py

# 3. Confirmar con "SI"
```

### **Ejemplo de Salida:**

```
docs/features/VELA-561/
├── README.md                          # Doc de Historia
├── TASK-001.md                        # Doc de Subtask 1
├── TASK-002.md                        # Doc de Subtask 2
└── ...

docs/architecture/
├── ADR-001-decidir-lenguaje.md       # Architecture Decision
└── ...

src/
├── feature-implementation.py          # Código fuente
└── ...

tests/unit/
├── test_feature-implementation.py     # Tests unitarios
└── ...
```

---

## 📚 Documentación

### **Lee primero:**
1. **`.github/CONTRIBUTING.md`** - Guía completa de desarrollo ⭐⭐⭐
2. **`RESUMEN_DE_CAMBIOS.md`** - Qué cambió y por qué
3. **`CHANGELOG.md`** - Historial de cambios

### **Scripts disponibles:**
- `develop_historia_v2.py` ⭐ - Automatización con entregables reales (USAR ESTE)
- `develop_historia.py` - Script antiguo (OBSOLETO)

---

## 📋 Características Originales (Importación Inicial)

Scripts que fueron usados para la importación inicial del backlog:

- ✅ Crea jerarquía completa: Epics → Stories → Tasks
- ✅ Establece dependencias entre tareas (Issue Links)
- ✅ Crea sprints automáticamente
- ✅ Asigna equipos y etiquetas
- ✅ Convierte estimaciones (horas → story points)
- ✅ Modo dry-run para probar sin crear issues
- ✅ Log detallado de todas las operaciones
- ✅ Manejo de errores robusto

## 🚀 Instalación

### 1. Requisitos

```bash
# Python 3.8 o superior
python --version

# Instalar dependencias
pip install requests
```

### 2. Configuración

```bash
# Copiar archivo de ejemplo
copy config.example.py config.py

# Editar config.py con tus credenciales
notepad config.py
```

### 3. Obtener API Token de Jira

1. Ve a: https://id.atlassian.com/manage-profile/security/api-tokens
2. Click en "Create API token"
3. Dale un nombre: "Vela Backlog Import"
4. Copia el token generado
5. Pégalo en `config.py` en la variable `JIRA_API_TOKEN`

### 4. Encontrar Custom Field IDs

Los custom field IDs varían por instancia de Jira. Para encontrar los tuyos:

```bash
# Windows (PowerShell)
$auth = [Convert]::ToBase64String([Text.Encoding]::ASCII.GetBytes("tu-email@ejemplo.com:tu-api-token"))
$headers = @{Authorization = "Basic $auth"}
Invoke-RestMethod -Uri "https://tu-dominio.atlassian.net/rest/api/3/field" -Headers $headers | ConvertTo-Json -Depth 10 > fields.json

# Ver fields.json y buscar:
# - "Epic Name" → customfield_10011
# - "Epic Link" → customfield_10014
# - "Story Points" → customfield_10016
# - "Sprint" → customfield_10020
```

Edita estos valores en `config.py`.

## 📖 Uso

### Modo Dry Run (Recomendado primero)

Prueba la importación sin crear issues reales:

```bash
python jira_importer.py --dry-run
```

Output esperado:
```
🔍 Verificando conexión con Jira...
✅ Conectado a Jira como: Tu Nombre (tu@email.com)

🔍 Verificando proyecto: VELA
✅ Proyecto encontrado: Vela Language Development

📄 Leídas 310 tareas del CSV

📘 FASE 1: Creando Epics...
[DRY RUN] Crearía Epic: VELA-1 - EPIC-00A: Critical Decisions (Phase 0)
[DRY RUN] Crearía Epic: VELA-2 - EPIC-00B: Formal Specifications (Phase 0)
...

✅ DRY RUN COMPLETADO (no se crearon issues reales)

📊 Resumen:
   - Epics creados: 50
   - Stories creados: 85
   - Tasks creados: 310
   - Sprints creados: 46
   - Links creados: 245
```

### Importación Real

Una vez verificado el dry-run:

```bash
python jira_importer.py
```

El script te pedirá confirmación:

```
⚠️  ADVERTENCIA: Esto creará cientos de issues en Jira
¿Estás seguro de continuar? (escribe 'SI' para confirmar): SI
```

### Usar archivo CSV diferente

```bash
python jira_importer.py --csv ruta/a/otro-roadmap.csv
```

## 📊 Estructura de Issues Creados

```
Phase 0 (Sprint 0)
├── 📘 EPIC-00A: Critical Decisions
│   ├── 📗 US-00A: Como líder técnico...
│   │   ├── 📙 TASK-000A: Decidir lenguaje de implementación
│   │   ├── 📙 TASK-000B: Definir arquitectura del build system
│   │   └── ...
│   └── ...
├── 📘 EPIC-00B: Formal Specifications
│   └── ...
...

Vela 1.0 (Sprints 1-40)
├── 📘 EPIC-01: Language Core
├── 📘 EPIC-02: Type System
├── 📘 EPIC-03: Reactive System
└── ...

Vela 2.0 (Sprints 42-46)
└── ...

Vela 3.0 (Future)
└── ...
```

### Metadata de Issues

Cada issue incluirá:

- **Summary**: ID de la tarea (ej: "TASK-000A: Decidir lenguaje")
- **Description**: Descripción completa del CSV
- **Labels**: Team, Milestone, Sprint
- **Priority**: P0 → Highest, P1 → High, P2 → Medium
- **Story Points**: Horas ÷ 8 (ej: 40h = 5 SP)
- **Sprint**: Asignado automáticamente si existe board
- **Dependencies**: Links tipo "Blocks" entre tareas

## 🔧 Troubleshooting

### Error: "401 Unauthorized"

```
❌ Error de autenticación: Credenciales inválidas
```

**Solución**: Verifica tu email y API token en `config.py`

### Error: "404 Project not found"

```
❌ Proyecto 'VELA' no encontrado
```

**Solución**: Verifica que `PROJECT_KEY` en `config.py` coincida con tu proyecto en Jira

### Error: "400 Bad Request" al crear tasks

```
❌ Error creando Task TASK-001: 400
```

**Soluciones**:
1. Verifica que los custom field IDs sean correctos
2. Verifica que tu proyecto permite crear subtasks (Tasks bajo Stories)
3. Ejecuta con `--dry-run` para ver qué campos fallan

### Error: Custom field not found

```
❌ Field 'customfield_10016' does not exist
```

**Solución**: Ejecuta el script `find_fields.py` para encontrar tus field IDs correctos

### Rate Limiting (429 Too Many Requests)

El script incluye delays de 0.3s entre requests. Si recibes errores 429:

1. Aumenta el delay en `time.sleep(0.3)` a `time.sleep(1.0)`
2. Ejecuta en horarios de menor carga

### Issues no se vinculan a Epic

Si las Stories no aparecen bajo el Epic:

1. Verifica que `FIELD_EPIC_LINK` sea correcto
2. Algunas versiones de Jira usan jerarquía de issues en lugar de Epic Link
3. Considera usar `"parent": {"key": epic_key}` en lugar de Epic Link

## 📝 Logs

Cada ejecución genera un log detallado:

```
import_log_20251130_153045.txt
```

Contiene:
- Timestamp de cada operación
- Keys de issues creados
- Errores detallados
- Resumen final

## 🔄 Rollback

Si necesitas eliminar los issues creados:

```bash
python rollback.py --log-file import_log_20251130_153045.txt
```

⚠️ **CUIDADO**: Esto eliminará permanentemente todos los issues creados.

## 🎯 Flujo Recomendado

1. **Configurar** `config.py` con tus credenciales
2. **Dry run** para verificar: `python jira_importer.py --dry-run`
3. **Revisar** el log generado
4. **Ajustar** custom fields si es necesario
5. **Importar** real: `python jira_importer.py`
6. **Verificar** en Jira que todo se creó correctamente
7. **Guardar** el log por si necesitas rollback

## 📚 Archivos

- `jira_importer.py` - Script principal de importación
- `config.example.py` - Plantilla de configuración
- `config.py` - Tu configuración (no subir a Git)
- `README.md` - Esta documentación
- `requirements.txt` - Dependencias Python
- `find_fields.py` - Helper para encontrar custom fields
- `rollback.py` - Script de rollback (opcional)

## 🆘 Soporte

Si encuentras problemas:

1. Revisa los logs generados
2. Ejecuta con `--dry-run` primero
3. Verifica permisos en Jira (necesitas poder crear issues)
4. Verifica que tu plan de Jira soporte la API (Free/Standard/Premium)

## ⚠️ Limitaciones

- Requiere Jira Cloud (versión 3 de API)
- Custom field IDs deben configurarse manualmente
- Sprints solo se crean si existe un board Scrum
- Rate limiting de Jira puede requerir delays mayores
- Algunos planes de Jira tienen límites de issues

## 📄 Licencia

Parte del proyecto Vela Language - Uso interno del equipo de desarrollo
