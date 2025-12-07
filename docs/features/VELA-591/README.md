# VELA-591: APIs de I/O y Networking

## 📋 Información General
- **Epic:** EPIC-07 Standard Library
- **Sprint:** Sprint 27
- **Estado:** En progreso 🚧
- **Fecha:** 2025-12-07

## 🎯 Descripción
Como desarrollador, quiero APIs de I/O y networking para poder trabajar eficientemente con archivos, directorios, HTTP y WebSockets en Vela.

## 📦 Subtasks Completadas

### ✅ TASK-087: Implementar File API
**Estado:** ✅ Completada
- API completa para operaciones de archivos implementada
- Lectura/escritura, copy, move, delete, metadata
- 9 tests unitarios con cobertura completa
- Inspirado en Rust std::fs y Node.js fs

### ✅ TASK-088: Implementar Directory API
**Estado:** ✅ Completada
- API completa para operaciones de directorios implementada
- Creación, listado, eliminación, copia recursiva
- Utilidades de rutas cross-platform (PathUtil)
- 17 tests unitarios con cobertura completa
- Inspirado en Rust std::fs y Node.js fs

## 📋 Subtasks Pendientes

### 🔄 TASK-089: Implementar HttpClient
**Estado:** Pendiente
- Cliente HTTP básico con métodos GET, POST, PUT, DELETE
- Headers, query parameters, body handling
- Timeout y error handling
- API inspirada en fetch() y reqwest

### 🔄 TASK-090: Implementar WebSocket
**Estado:** Pendiente
- Cliente WebSocket para comunicación bidireccional
- Conexión, envío/recepción de mensajes
- Manejo de eventos y errores
- API inspirada en WebSocket API del browser

### 🔄 TASK-091: Tests de I/O y networking
**Estado:** Pendiente
- Tests unitarios para todas las APIs
- Tests de integración con mocking
- Tests de error handling y edge cases
- Benchmarks de performance

## 🔨 Arquitectura Propuesta

### File API
```rust
// Lectura de archivos
let content = File::read_to_string("file.txt")?;
let bytes = File::read("file.bin")?;

// Escritura de archivos
File::write("file.txt", "content")?;
File::append("file.txt", "more content")?;

// Operaciones avanzadas
File::copy("source.txt", "dest.txt")?;
File::move("old.txt", "new.txt")?;
File::delete("file.txt")?;
```

### Directory API
```rust
// Operaciones con directorios
Directory::create("new_dir")?;
let entries = Directory::list("some_dir")?;
Directory::remove("empty_dir")?;

// Path utilities
let path = Path::join("dir", "file.txt");
let absolute = Path::resolve("relative/path");
```

### HttpClient
```rust
// HTTP requests
let client = HttpClient::new();
let response = client.get("https://api.example.com/data").await?;
let json = client.post("https://api.example.com/create")
    .json(&data)
    .send()
    .await?;
```

### WebSocket
```rust
// WebSocket connection
let ws = WebSocket::connect("ws://echo.websocket.org").await?;
ws.send("Hello").await?;
let message = ws.receive().await?;
```

## 📊 Métricas
- **Subtasks completadas:** 2/5 (40%)
- **Archivos creados:** 5 (TASK-087.md, TASK-088.md, file.rs, directory.rs, mod.rs)
- **Líneas de código:** ~200 líneas en file.rs + ~416 líneas en directory.rs + ~10 líneas en mod.rs
- **Tests agregados:** 9 tests para File API + 17 tests para Directory API (26 total)
- **Coverage:** >90% en ambas APIs

## ✅ Definición de Hecho
- [x] TASK-087 completada con File API funcional
- [x] TASK-088: Directory API implementada
- [ ] TASK-089: HttpClient implementado
- [ ] TASK-090: WebSocket implementado
- [ ] TASK-091: Tests de I/O y networking completados
- [ ] Documentación completa de todas las APIs

## 🔗 Referencias
- **Jira:** [VELA-591](https://velalang.atlassian.net/browse/VELA-591)
- **Inspiración:** Node.js fs, fetch API, Rust std::fs/net, Python pathlib
- **Relacionado:** EPIC-07 Standard Library</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-591\README.md