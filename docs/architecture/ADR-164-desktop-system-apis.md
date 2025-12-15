# ADR-164: Desktop System APIs Architecture

## Estado
✅ Aceptado

## Fecha
2025-01-30

## Contexto
Necesitamos implementar APIs del sistema nativo para aplicaciones desktop Vela, incluyendo:
- File system operations (read, write, list, watch)
- Process management (spawn, kill, communicate)
- System information (OS version, hardware info)
- Network APIs (HTTP client, WebSocket)

## Decisión
Implementar system APIs usando crates Rust nativos en lugar de bindings C++, priorizando seguridad, async operations y cross-platform compatibility.

### 1. File System APIs
**Crate:** `tokio::fs` + `std::fs`
- Async operations con tokio para I/O no bloqueante
- Path handling con `std::path`
- Metadata operations con `std::fs::Metadata`

### 2. Process Management
**Crate:** `tokio::process`
- Async process spawning y management
- Cross-platform process handling
- Signal handling y communication

### 3. Network APIs
**HTTP Client:** `reqwest` v0.11
- Async HTTP requests con tokio
- JSON serialization con serde
- Connection pooling automático

**WebSocket:** `tokio-tungstenite` v0.20
- Async WebSocket connections
- Native TLS support
- Integration con tokio ecosystem

### 4. System Information
**Crate:** `sysinfo` v0.30
- Cross-platform system information
- Hardware info (CPU, memory, disks)
- Process information
- Network interfaces

## Consecuencias

### Positivas
- **Seguridad:** APIs Rust nativas sin FFI unsafe
- **Performance:** Async operations no bloqueantes
- **Mantenibilidad:** Código Rust puro, mejor debugging
- **Cross-platform:** Implementaciones nativas por plataforma
- **Ecosystem:** Integración perfecta con tokio

### Negativas
- **Bundle size:** Dependencias adicionales (~5-10MB)
- **Build time:** Compilación de crates adicionales
- **Complejidad:** Manejo de async/await en APIs públicas

## Alternativas Consideradas

### 1. Bindings C++ (como estaba planeado)
**Rechazada porque:**
- Unsafe FFI code aumenta riesgo de crashes
- Complejidad de mantenimiento de bindings
- Dependencia de implementación C++

### 2. std:: only (sin crates externas)
**Rechazada porque:**
- Falta async operations
- Limitada funcionalidad de red
- No cross-platform system info

### 3. Custom HTTP/WebSocket implementation
**Rechazada porque:**
- Desarrollo time mucho mayor
- Mayor riesgo de bugs de seguridad
- Reinventar la rueda

## Implementación
Ver código en: `runtime/desktop/src/system_apis.rs`

### API Structure
```rust
pub mod fs {
    pub async fn read_file(path: impl AsRef<Path>) -> Result<Vec<u8>>
    pub async fn write_file(path: impl AsRef<Path>, data: &[u8]) -> Result<()>
    // ... más operaciones
}

pub mod process {
    pub async fn spawn(cmd: &str, args: &[&str]) -> Result<ChildProcess>
    // ... process management
}

pub mod net {
    pub struct HttpClient { /* reqwest-based */ }
    pub struct WebSocketClient { /* tungstenite-based */ }
}

pub mod system {
    pub fn get_info() -> SystemInfo // sysinfo-based
}
```

## Referencias
- Jira: [VELA-1173](https://velalang.atlassian.net/browse/VELA-1173)
- Documentación: `docs/features/VELA-1173/TASK-164.md`
- Código: `runtime/desktop/src/system_apis.rs`