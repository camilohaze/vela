# ADR-090: Diseño de la API WebSocket

## Estado
✅ Aceptado

## Fecha
2025-12-07

## Contexto
Necesitamos implementar un cliente WebSocket para Vela que permita comunicación bidireccional en tiempo real. La API debe ser simple pero poderosa, manejando conexiones, envío/recepción de mensajes, eventos de conexión y desconexión, y errores de manera robusta.

El cliente debe:
- Conectar a servidores WebSocket (ws:// y wss://)
- Enviar y recibir mensajes de texto y binarios
- Manejar eventos de conexión (open, close, error)
- Soporte para ping/pong automático
- Configuración de timeouts y reconnection
- API asíncrona con futures/promises
- Manejo de errores específico para WebSocket

## Decisión
Implementaremos una API WebSocket inspirada en la WebSocket API del browser pero con mejoras para casos de uso empresarial:

### API Principal
```vela
// Conexión básica
let ws = WebSocket::connect("ws://echo.websocket.org").await?;
ws.send("Hello WebSocket!").await?;

// Event-driven con callbacks
ws.on_message(|message| {
    println!("Received: {}", message);
});

ws.on_close(|code, reason| {
    println!("Connection closed: {} - {}", code, reason);
});

// Async/await con streams
while let Some(message) = ws.receive().await? {
    match message {
        Message::Text(text) => println!("Text: {}", text),
        Message::Binary(data) => println!("Binary: {} bytes", data.len()),
        Message::Close { code, reason } => break,
    }
}
```

### Características Clave
1. **Event-Driven**: Callbacks para eventos (open, message, close, error)
2. **Stream-Based**: Iteradores async para recepción de mensajes
3. **Type Safety**: Enums para tipos de mensaje (Text, Binary, Close, Ping, Pong)
4. **Error Handling**: Errores específicos (ConnectionFailed, Timeout, ProtocolError)
5. **Configurable**: Timeouts, headers, subprotocolos
6. **Auto-Reconnection**: Opcional con backoff exponencial

### Estructuras
- `WebSocket`: Cliente principal con métodos de conexión
- `WebSocketConnection`: Conexión activa con métodos send/receive
- `Message`: Enum para tipos de mensaje (Text, Binary, Close, Ping, Pong)
- `WebSocketError`: Tipos de error específicos
- `WebSocketConfig`: Configuración de conexión (timeout, headers, etc.)

### Estados de Conexión
- `Connecting`: Estableciendo conexión
- `Connected`: Conexión activa
- `Closing`: Cerrando conexión
- `Closed`: Conexión cerrada

## Consecuencias

### Positivas
- ✅ API familiar similar a WebSocket browser API
- ✅ Soporte completo para comunicación bidireccional
- ✅ Type safety con enums para mensajes
- ✅ Manejo robusto de errores y estados
- ✅ Configurable y extensible
- ✅ Inspirado en las mejores prácticas de la industria

### Negativas
- ❌ Complejidad mayor que una API minimalista
- ❌ Dependencia de crates async (tokio-tungstenite)
- ❌ Curva de aprendizaje para usuarios nuevos
- ❌ Manejo de estados de conexión más complejo

## Alternativas Consideradas

### 1. API Minimalista (Rechazada)
```vela
// Muy simple pero limitado
let ws = websocket_connect("ws://example.com");
ws.send("hello");
let response = ws.receive();
```
**Rechazada porque:** No soporta eventos, tipos de mensaje, errores específicos, configuración.

### 2. API Similar a Socket.IO (Rechazada)
```vela
let socket = io("ws://example.com");
socket.on("connect", () => { ... });
socket.on("message", (data) => { ... });
socket.emit("event", data);
```
**Rechazada porque:** Muy específico para Socket.IO, no estándar WebSocket puro.

### 3. API Similar a tokio-tungstenite (Rechazada)
```vela
let (ws_stream, _) = connect_async(url).await?;
let (write, read) = ws_stream.split();
```
**Rechazada porque:** Demasiado low-level, requiere manejo manual de streams, no user-friendly.

## Referencias
- **Jira:** [TASK-090](https://velalang.atlassian.net/browse/TASK-090)
- **Historia:** [VELA-591](https://velalang.atlassian.net/browse/VELA-591)
- **Inspiración:**
  - Browser WebSocket API
  - tokio-tungstenite crate
  - websockets crate (Python)
  - Socket.IO client
- **Documentación:** Ver código en `src/websocket.rs`</content>
<parameter name="filePath">c:\Users\cristian.naranjo\Downloads\Vela\docs\architecture\ADR-090-websocket-api.md