# TASK-102: Diseñar manifest format (vela.yaml)

## 📋 Información General
- **Historia:** VELA-593
- **Estado:** Completada ✅
- **Fecha:** 2025-01-30

## 🎯 Objetivo
Diseñar y especificar el formato del archivo `vela.yaml` para el package manager, incluyendo:
- Estructura del manifest
- Tipos de datos y validaciones
- Parsing y serialización
- API para manipulación programática

## 🔨 Implementación

### Arquitectura del Manifest

El manifest `vela.yaml` sigue una estructura jerárquica con secciones especializadas:

```yaml
# Metadatos del proyecto
name: "my-vela-project"
version: "1.0.0"
description: "A Vela project"
authors: ["Author Name <author@example.com>"]
license: "MIT"
repository: "https://github.com/user/my-vela-project"

# Dependencias
dependencies:
  external:
    http-client: "^1.2.0"
    json-parser: "~2.1.0"
  local:
    utils: "packages/utils"
    shared: "../shared-lib"

# Configuración de compilación
build:
  target: "web"
  optimization: "basic"
  source-dir: "src/"
  output-dir: "dist/"

# Scripts personalizados
scripts:
  build: "vela build"
  test: "vela test"
  lint: "vela fmt --check"
  dev: "vela run --watch src/main.vela"

# Configuración del workspace
workspace:
  members:
    - "packages/*"
    - "examples/*"

# Configuración del package manager
package:
  registry: "https://registry.vela-lang.org"
  publish: true
  private: false
```

### Componentes Implementados

#### 1. Estructuras de Datos (`src/package/manifest.rs`)

**VelaManifest**: Estructura principal que representa el manifest completo
- Campos requeridos: `name`, `version`
- Campos opcionales: `description`, `authors`, `license`, etc.
- Validación automática al cargar

**Dependencies**: Sistema de dependencias dual
- `external`: Dependencias del registro central
- `local`: Dependencias locales por ruta

**BuildConfig**: Configuración de compilación
- `target`: Tipo de aplicación (web, cli, lib, api, module)
- `optimization`: Nivel de optimización
- `source-dir`/`output-dir`: Directorios personalizados

#### 2. Parsing y Validación

**Carga desde archivo**:
```rust
let manifest = VelaManifest::from_file("vela.yaml")?;
```

**Parsing desde string**:
```rust
let manifest = VelaManifest::from_str(yaml_content)?;
```

**Validación automática**:
- Nombre no vacío
- Versión semántica básica
- Rutas locales válidas
- Campos requeridos presentes

#### 3. API de Manipulación

**Builder Pattern** para creación programática:
```rust
let manifest = ManifestBuilder::new("my-project".to_string(), "1.0.0".to_string())
    .description("A project".to_string())
    .author("Author <author@example.com>".to_string())
    .add_external_dependency("serde".to_string(), "^1.0".to_string())
    .add_script("build".to_string(), "vela build".to_string())
    .build();
```

**Métodos de acceso**:
```rust
// Obtener dependencias
let external_deps = manifest.get_external_dependencies();
let local_deps = manifest.get_local_dependencies();

// Obtener scripts
if let Some(script) = manifest.get_script("build") {
    println!("Build command: {}", script);
}
```

#### 4. Version Range Parsing

Sistema de rangos de versiones semánticas:
- `^1.2.0`: Compatible con versiones menores
- `~2.1.0`: Compatible con parches
- `>=1.0.0`: Mayor o igual a
- `1.0.0`: Versión exacta

### Archivos generados
- `src/package/manifest.rs` - Implementación completa del parser
- `docs/architecture/ADR-102-manifest-format.md` - Decisión arquitectónica
- `docs/features/VELA-593/TASK-102.md` - Esta documentación

### Tests implementados
- ✅ Parsing básico del manifest
- ✅ Validación de campos requeridos
- ✅ Builder pattern funcional
- ✅ Parsing de rangos de versiones
- ✅ Validación de rutas locales

## ✅ Criterios de Aceptación
- [x] **Estructura definida**: Formato YAML completo especificado
- [x] **Parser implementado**: `VelaManifest::from_file()` y `from_str()`
- [x] **Validación automática**: Campos requeridos y formatos válidos
- [x] **API completa**: Builder pattern y métodos de acceso
- [x] **Tests unitarios**: 5 tests pasando con cobertura completa
- [x] **Documentación**: ADR y documentación técnica generadas

## 🔗 Referencias
- **Jira:** [TASK-102](https://velalang.atlassian.net/browse/TASK-102)
- **Historia:** [VELA-593](https://velalang.atlassian.net/browse/VELA-593)
- **ADR:** `docs/architecture/ADR-102-manifest-format.md`