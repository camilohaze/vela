# ADR-162: Arquitectura del Desktop Runtime

## Estado
✅ Aceptado

## Fecha
2025-12-15

## Contexto
Necesitamos implementar un runtime nativo para aplicaciones desktop en Vela que soporte Windows, macOS y Linux con performance comparable a aplicaciones nativas. El runtime debe integrar con el sistema de widgets existente de Vela mientras proporciona acceso a APIs nativas del sistema operativo.

## Decisión
Implementaremos un desktop runtime basado en C++ con las siguientes características:

### Arquitectura Elegida
```
┌─────────────────────────────────────────────────────────────┐
│                    Desktop Application                       │
├─────────────────────────────────────────────────────────────┤
│  ┌─────────────────┐    ┌──────────────────┐    ┌─────────┐  │
│  │ Vela Runtime    │───▶│ Desktop Bridge   │───▶│ Skia    │  │
│  │ (Rust)          │    │ (FFI/C++)        │    │ Renderer │  │
│  └─────────────────┘    └──────────────────┘    └─────────┘  │
├─────────────────────────────────────────────────────────────┤
│                Native OS (Windows/macOS/Linux)              │
└─────────────────────────────────────────────────────────────┘
```

### Componentes Principales

#### 1. DesktopRenderEngine (C++)
- Motor de renderizado principal
- Gestiona el render loop a 60 FPS con VSync
- Coordina entre Vela runtime y Skia renderer
- Maneja el ciclo de vida de la aplicación

#### 2. VelaDesktopBridge (FFI)
- Puente entre Rust y C++ usando FFI seguro
- Serialización/deserialización de VDOM
- Gestión de memoria compartida
- Thread safety con locks apropiados

#### 3. Skia Integration
- Renderer 2D acelerado por hardware
- Soporte completo para gráficos vectoriales
- Text rendering con fuentes del sistema
- Image loading y caching

#### 4. Platform-Specific Layers
- **Windows**: Win32 API + DirectX
- **macOS**: Cocoa/AppKit + Metal
- **Linux**: X11/Wayland + Vulkan

### APIs del Sistema
- **File System**: Lectura/escritura de archivos, watchers
- **Process Management**: Spawn, kill, comunicación con procesos
- **Network**: HTTP client, WebSocket, TCP/UDP
- **System Info**: OS version, hardware specs, environment
- **Clipboard**: Get/set clipboard content
- **Notifications**: Desktop notifications
- **Power Management**: Sleep, wake, battery status

## Consecuencias

### Positivas
- **Performance nativa**: 60 FPS garantizado con Skia
- **Compatibilidad total**: APIs nativas del sistema operativo
- **Reutilización de código**: Widgets Vela existentes funcionan en desktop
- **Distribución fácil**: Binarios standalone sin dependencias externas
- **Seguridad**: Gestión de memoria segura con RAII

### Negativas
- **Complejidad de implementación**: Tres plataformas diferentes
- **Mantenimiento**: Actualizaciones de APIs nativas por plataforma
- **Bundle size**: Binarios más grandes que web apps
- **Build complexity**: Cross-compilation para múltiples targets

## Alternativas Consideradas

### 1. Electron-based (Rechazada)
- **Pros**: Fácil de implementar, JavaScript ecosystem
- **Cons**: Alto consumo de memoria, performance limitada
- **Razón**: No cumple con requerimiento de "performance comparable a nativas"

### 2. Qt-based (Rechazada)
- **Pros**: Multiplataforma madura, buen tooling
- **Cons**: Dependencia externa pesada, licensing concerns
- **Razón**: Queremos control total del stack y zero dependencies

### 3. Flutter-style (Rechazada)
- **Pros**: Similar a nuestro approach actual
- **Cons**: Complejidad de mantener dos engines (web + desktop)
- **Razón**: Mejor mantener consistencia con un solo renderer

## Implementación
Ver código en: `runtime/desktop/`

## Referencias
- Jira: [VELA-1173](https://velalang.atlassian.net/browse/VELA-1173)
- Documentación: [Desktop Runtime Architecture](docs/architecture/desktop-runtime.md)</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-162-desktop-runtime-architecture.md