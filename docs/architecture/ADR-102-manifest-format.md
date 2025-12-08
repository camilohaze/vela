# ADR-102: Formato del Manifest vela.yaml

## Estado
✅ Aceptado

## Fecha
2025-01-30

## Contexto
Necesitamos un formato de manifest para el package manager de Vela que permita:
- Definir metadatos del proyecto (nombre, versión, descripción)
- Especificar dependencias con versiones
- Configurar opciones de compilación
- Definir scripts y comandos personalizados
- Ser extensible para futuras funcionalidades

El formato debe ser:
- Humano-legible y editable
- Máquina-parseable
- Compatible con herramientas existentes
- Seguro y validable

## Decisión
Adoptar YAML como formato base para `vela.yaml` con la siguiente estructura:

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
  # Dependencias externas (de registro)
  external:
    http-client: "^1.2.0"
    json-parser: "~2.1.0"

  # Dependencias locales (rutas relativas)
  local:
    utils: "packages/utils"
    shared: "../shared-lib"

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
  # Dependencias externas (de registro)
  external:
    http-client: "^1.2.0"
    json-parser: "~2.1.0"

  # Dependencias locales (rutas relativas)
  local:
    utils: "packages/utils"
    shared: "../shared-lib"

# Configuración de compilación
build:
  target: "web-app"  # web-app, mobile-app, desktop-app, service, library, cli-tool
  platform: "web"    # web, android, ios, windows, macos, linux
  optimization: "basic"  # none, basic, aggressive, maximum
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

## Consecuencias

### Positivas
- **Legibilidad**: YAML es fácil de leer y escribir para humanos
- **Ecosistema**: Amplio soporte en herramientas y lenguajes
- **Extensibilidad**: Fácil agregar nuevos campos sin breaking changes
- **Validación**: Esquemas JSON Schema para validación automática
- **Compatibilidad**: Similar a package.json, Cargo.toml, etc.

### Negativas
- **Indentación sensible**: Errores de indentación pueden causar problemas
- **Comentarios limitados**: Solo comentarios de línea simple
- **Parsing complejo**: Requiere librería YAML (vs JSON simple)

## Alternativas Consideradas

### 1. JSON (Rechazado)
```json
{
  "name": "my-project",
  "dependencies": {
    "http-client": "^1.2.0"
  }
}
```
**Razones de rechazo:**
- Menos legible para humanos
- No soporta comentarios
- Más verboso para configuración compleja

### 2. TOML (Rechazado)
```toml
[package]
name = "my-project"
version = "1.0.0"

[dependencies]
http-client = "^1.2.0"
```
**Razones de rechazo:**
- Menos conocido que YAML
- Sintaxis más rígida
- Menos herramientas de edición disponibles

### 3. HCL (HashiCorp Configuration Language) (Rechazado)
```hcl
project {
  name = "my-project"
  dependencies = {
    http-client = "^1.2.0"
  }
}
```
**Razones de rechazo:**
- Muy específico de HashiCorp
- Ecosistema limitado
- Curva de aprendizaje adicional

### 4. Custom DSL (Rechazado)
```vela
project "my-project" {
    version "1.0.0"
    dependency "http-client" "^1.2.0"
}
```
**Razones de rechazo:**
- Requiere parser custom complejo
- No reutiliza herramientas existentes
- Mayor mantenimiento

## Implementación
Ver código en: `src/package/manifest.rs`

## Referencias
- Jira: [TASK-102](https://velalang.atlassian.net/browse/TASK-102)
- Historia: [VELA-593](https://velalang.atlassian.net/browse/VELA-593)
- Documentación: `docs/features/VELA-593/TASK-102.md`