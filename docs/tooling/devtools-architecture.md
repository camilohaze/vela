# Arquitectura de Vela DevTools

**Historia:** VELA-562 (US-00C)  
**Subtask:** TASK-000M  
**Fecha:** 2025-11-30  
**Estado:** ✅ Completado

---

## 📋 Resumen Ejecutivo

Este documento define la arquitectura de **Vela DevTools**, una suite de herramientas de debugging y profiling embebida que se ejecuta en el navegador. Inspirado en React DevTools, Flutter DevTools y Chrome DevTools, proporciona introspección profunda de aplicaciones Vela en runtime.

---

## 1. Componentes Principales

### **1.1. UI Inspector**
- Visualización de árbol de widgets (tree view)
- Live editing de propiedades
- Layout debugging (bounding boxes, padding, margin)

### **1.2. Signal Graph Visualizer**
- Grafo de dependencias reactivas (signals → computed → effects)
- Tracking de valores en tiempo real
- Timeline de actualizaciones

### **1.3. Performance Profiler**
- CPU profiling (flamegraph)
- Memory profiling (heap snapshots)
- Network inspector (HTTP requests)

---

## 2. Architecture Overview

```
┌────────────────────────────────────────────────────────────────┐
│                     Vela Application                            │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                  Vela Runtime                             │  │
│  │  - Widget tree                                            │  │
│  │  - Signal graph                                           │  │
│  │  - Memory allocator                                       │  │
│  └───────────────────┬──────────────────────────────────────┘  │
│                      │ (WebSocket)                              │
│  ┌───────────────────v──────────────────────────────────────┐  │
│  │              DevTools Agent (Rust)                        │  │
│  │  - Introspection hooks                                    │  │
│  │  - Serialization (widget tree → JSON)                     │  │
│  │  - Live editing backend                                   │  │
│  └───────────────────┬──────────────────────────────────────┘  │
└────────────────────────┼────────────────────────────────────────┘
                         │
                         │ WebSocket (ws://localhost:9229)
                         │
┌────────────────────────v─────────────────────────────────────┐
│                   DevTools Server (Rust)                      │
│  - WebSocket server (tokio-tungstenite)                       │
│  - Static file serving (DevTools UI)                          │
│  - Protocol handling (commands, responses)                    │
└────────────────────────┬─────────────────────────────────────┘
                         │ HTTP (localhost:9229)
                         v
┌──────────────────────────────────────────────────────────────┐
│                   Web Browser                                 │
│  ┌────────────────────────────────────────────────────────┐  │
│  │           DevTools UI (TypeScript + React)             │  │
│  │  ┌──────────────┬─────────────┬──────────────────┐     │  │
│  │  │ UI Inspector │ Signal Graph│ Perf Profiler    │     │  │
│  │  └──────────────┴─────────────┴──────────────────┘     │  │
│  └────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

## 3. UI Inspector

### **3.1. Widget Tree Visualization**

**Funcionalidad:**
- Tree view de todos los widgets (colapsable)
- Highlight widget on hover (outline en app)
- Select widget → mostrar properties panel

**Ejemplo de UI:**
```
📦 MyApp
 ├─ 📦 AppBar
 │   ├─ 📄 Text: "My App"
 │   └─ 🔘 IconButton
 ├─ 📦 Body
 │   ├─ 📦 ListView
 │   │   ├─ 📦 ListItem (x20)
 │   │   │   ├─ 📄 Text
 │   │   │   └─ 🔘 Button
```

**Data structure (JSON over WebSocket):**
```json
{
  "type": "widget_tree",
  "root": {
    "id": "0x1a2b3c",
    "widget_type": "MyApp",
    "children": [
      {
        "id": "0x1a2b3d",
        "widget_type": "AppBar",
        "props": {
          "title": "My App",
          "backgroundColor": "#2196F3"
        },
        "children": [...]
      }
    ]
  }
}
```

---

### **3.2. Live Editing**

**Funcionalidad:**
- Editar propiedades en DevTools → actualizar app en tiempo real
- Hot reload de valores (sin recompilar)

**Flujo:**
1. Usuario edita prop en DevTools: `backgroundColor = "#FF0000"`
2. DevTools envía comando WebSocket:
   ```json
   {
     "command": "update_prop",
     "widget_id": "0x1a2b3d",
     "prop_name": "backgroundColor",
     "new_value": "#FF0000"
   }
   ```
3. Agent actualiza prop en runtime
4. Widget se re-renderiza automáticamente (reactivo)

**Implementación (Rust):**
```rust
// DevTools Agent
fn handle_update_prop(widget_id: WidgetId, prop_name: &str, new_value: Value) {
    let widget = WIDGET_TREE.lock().find_widget(widget_id)?;
    widget.set_prop(prop_name, new_value);
    widget.mark_dirty();  // Trigger re-render
}
```

---

### **3.3. Layout Debugging**

**Funcionalidad:**
- Overlay de bounding boxes
- Visualizar padding, margin, border
- Highlight layout constraints violations

**UI Controls:**
```
[x] Show bounding boxes
[x] Show padding (green)
[x] Show margin (orange)
[ ] Show baseline grid
```

**Implementación:**
- Agent inyecta debug layer en renderer
- Dibuja overlays con colores semi-transparentes

---

## 4. Signal Graph Visualizer

### **4.1. Dependency Graph**

**Funcionalidad:**
- Grafo interactivo de signals, computed, effects
- Nodos: Signals (azul), Computed (verde), Effects (naranja)
- Edges: Dependencias (signal → computed → effect)

**Ejemplo de grafo:**
```
Signal<Int>: counter
      │
      ├──> Computed<String>: counterText = "Count: ${counter}"
      │         │
      │         └──> Effect: updateUI()
      │
      └──> Computed<Bool>: isEven = counter % 2 == 0
                │
                └──> Effect: toggleClass()
```

**Visualización:** D3.js force-directed graph.

---

### **4.2. Value Tracking**

**Funcionalidad:**
- Mostrar valor actual de cada signal/computed
- Actualizar en tiempo real cuando cambia
- Click en nodo → ver historial de valores

**UI:**
```
┌─────────────────────────────────────┐
│ Signal<Int>: counter                │
│ Value: 42                           │
│ History:                            │
│   10:23:45 → 41                     │
│   10:23:46 → 42                     │
│   10:23:47 → 42 (no change)         │
└─────────────────────────────────────┘
```

**Data structure:**
```json
{
  "type": "signal_snapshot",
  "signals": [
    {
      "id": "signal_1",
      "name": "counter",
      "type": "Signal<Int>",
      "value": 42,
      "dependents": ["computed_1", "computed_2"]
    }
  ]
}
```

---

### **4.3. Update Timeline**

**Funcionalidad:**
- Timeline horizontal de actualizaciones
- Ver orden de propagación (topological sort)
- Detectar computaciones redundantes

**UI:**
```
Timeline:
─────────────────────────────────────────────────────>
0ms    counter = 42
1ms    ├─> counterText = "Count: 42"
2ms    ├─> isEven = true
3ms    │   └─> Effect: toggleClass()
4ms    └─> Effect: updateUI()
```

**Implementación:**
- Agent captura eventos de actualización
- Envía batch de eventos cada 100ms

---

## 5. Performance Profiler

### **5.1. CPU Profiling (Flamegraph)**

**Funcionalidad:**
- Sampling profiler (captura stack traces cada 10ms)
- Flamegraph interactivo (click para zoom)
- Identificar hotspots (funciones lentas)

**UI:**
```
┌─────────────────────────────────────────────────────────────┐
│             main() [100%]                                    │
│  ┌───────────────────────────────────────────────────────┐  │
│  │       build_widget_tree() [60%]                        │  │
│  │  ┌──────────────────────┐   ┌────────────────────┐    │  │
│  │  │ parse_layout() [30%] │   │ render() [30%]     │    │  │
│  │  └──────────────────────┘   └────────────────────┘    │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

**Implementación:**
- `pprof` (Rust crate) para profiling
- Exportar flamegraph como SVG o JSON
- Enviar a DevTools UI

---

### **5.2. Memory Profiling**

**Funcionalidad:**
- Heap snapshots (capturar estado de memoria)
- Comparar snapshots (detectar leaks)
- Ver top allocations (qué objetos usan más memoria)

**UI:**
```
┌─────────────────────────────────────────────────────────┐
│ Heap Snapshot 1 (10:23:45)                              │
│ Total: 12.5 MB                                          │
│                                                         │
│ Top Allocations:                                        │
│ 1. Vec<Widget>       4.2 MB    [33%]                    │
│ 2. String            2.1 MB    [17%]                    │
│ 3. Signal<T>         1.5 MB    [12%]                    │
│ 4. HashMap<K,V>      1.0 MB    [8%]                     │
│ 5. Other             3.7 MB    [30%]                    │
└─────────────────────────────────────────────────────────┘
```

**Implementación:**
- Custom allocator con tracking
- Capturar metadata por allocation (type, size, stack trace)

---

### **5.3. Network Inspector**

**Funcionalidad:**
- Listar todas las HTTP requests
- Ver headers, body, status code
- Timeline de requests (waterfall)

**UI:**
```
┌──────────────────────────────────────────────────────────────────┐
│ Method  URL                        Status  Time     Size         │
├──────────────────────────────────────────────────────────────────┤
│ GET     /api/users                 200     120ms    2.3 KB       │
│ POST    /api/users/42              201     450ms    0.5 KB       │
│ GET     /api/posts                 200     80ms     5.1 KB       │
└──────────────────────────────────────────────────────────────────┘

Waterfall:
──────────────────────────────────────>
0ms      [=====] GET /api/users (120ms)
120ms         [=============] POST /api/users/42 (450ms)
570ms    [===] GET /api/posts (80ms)
```

**Implementación:**
- Hook en HTTP client stdlib
- Capturar request/response metadata
- Enviar a DevTools via WebSocket

---

## 6. DevTools Server

### **6.1. WebSocket Server**

**Framework:** `tokio-tungstenite`

**Endpoints:**
- `ws://localhost:9229/devtools`: WebSocket connection

**Protocol:**
```json
// Client → Server (command)
{
  "id": 1,
  "method": "get_widget_tree",
  "params": {}
}

// Server → Client (response)
{
  "id": 1,
  "result": {
    "root": { ... }
  }
}

// Server → Client (event)
{
  "method": "widget_updated",
  "params": {
    "widget_id": "0x1a2b3d",
    "props": { ... }
  }
}
```

---

### **6.2. Static File Serving**

**Purpose:** Servir DevTools UI (HTML, JS, CSS).

**Implementation:**
```rust
use axum::{Router, routing::get_service};
use tower_http::services::ServeDir;

let app = Router::new()
    .nest_service("/", get_service(ServeDir::new("devtools-ui/dist")));

axum::Server::bind(&"127.0.0.1:9229".parse().unwrap())
    .serve(app.into_make_service())
    .await
    .unwrap();
```

**URLs:**
- `http://localhost:9229/`: DevTools UI (HTML)
- `http://localhost:9229/assets/`: CSS, JS, images

---

## 7. DevTools UI (Frontend)

### **7.1. Tech Stack**

- **Framework:** React 18 (with hooks)
- **Visualization:** D3.js (signal graph), react-flamegraph (CPU profiling)
- **Styling:** Tailwind CSS
- **Build:** Vite

---

### **7.2. Component Structure**

```
devtools-ui/
├── src/
│   ├── App.tsx                  # Root component
│   ├── components/
│   │   ├── UIInspector/
│   │   │   ├── WidgetTree.tsx
│   │   │   ├── PropertiesPanel.tsx
│   │   │   └── LayoutOverlay.tsx
│   │   ├── SignalGraph/
│   │   │   ├── DependencyGraph.tsx
│   │   │   ├── ValueTracker.tsx
│   │   │   └── Timeline.tsx
│   │   └── Profiler/
│   │       ├── Flamegraph.tsx
│   │       ├── MemorySnapshot.tsx
│   │       └── NetworkInspector.tsx
│   ├── hooks/
│   │   ├── useWebSocket.ts      # WebSocket connection
│   │   └── useDevToolsState.ts  # Global state
│   └── types/
│       └── protocol.ts           # TypeScript types for protocol
```

---

### **7.3. WebSocket Client**

```typescript
// useWebSocket.ts
import { useEffect, useState } from 'react';

export function useWebSocket(url: string) {
  const [ws, setWs] = useState<WebSocket | null>(null);
  const [connected, setConnected] = useState(false);

  useEffect(() => {
    const socket = new WebSocket(url);
    
    socket.onopen = () => {
      setConnected(true);
      console.log('DevTools connected');
    };
    
    socket.onmessage = (event) => {
      const message = JSON.parse(event.data);
      handleMessage(message);
    };
    
    socket.onclose = () => {
      setConnected(false);
      console.log('DevTools disconnected');
    };
    
    setWs(socket);
    
    return () => socket.close();
  }, [url]);
  
  const sendCommand = (method: string, params: any) => {
    if (ws && connected) {
      ws.send(JSON.stringify({
        id: Math.random(),
        method,
        params,
      }));
    }
  };
  
  return { connected, sendCommand };
}
```

---

## 8. Agent Integration

### **8.1. Introspection Hooks**

**Runtime debe exponer APIs para introspección:**

```rust
// vela_runtime crate
pub trait DevToolsIntrospectable {
    /// Get current widget tree
    fn get_widget_tree(&self) -> WidgetTreeSnapshot;
    
    /// Get signal graph
    fn get_signal_graph(&self) -> SignalGraphSnapshot;
    
    /// Update widget property
    fn update_widget_prop(&mut self, widget_id: WidgetId, prop: &str, value: Value);
    
    /// Start CPU profiling
    fn start_profiling(&mut self);
    
    /// Stop CPU profiling and return flamegraph
    fn stop_profiling(&mut self) -> Flamegraph;
    
    /// Capture heap snapshot
    fn capture_heap_snapshot(&self) -> HeapSnapshot;
}
```

---

### **8.2. Conditional Compilation**

**DevTools solo en debug builds:**

```rust
#[cfg(debug_assertions)]
fn init_devtools() {
    let agent = DevToolsAgent::new();
    agent.start_server("127.0.0.1:9229");
}

#[cfg(not(debug_assertions))]
fn init_devtools() {
    // No-op in release
}
```

---

## 9. Security Considerations

### **9.1. Localhost Only**

- ✅ DevTools server solo escucha en `127.0.0.1` (no `0.0.0.0`)
- ✅ No exponer a internet (evitar remote debugging attacks)

---

### **9.2. Authentication (opcional)**

**Para producción debug:**
- Generar token random al iniciar
- Requerir token en WebSocket handshake

```rust
let token = generate_random_token();
println!("DevTools token: {}", token);

// En handshake:
if req.headers().get("Authorization") != Some(&token) {
    return Err("Unauthorized");
}
```

---

## 10. Performance Impact

### **10.1. Overhead Target**

| Métrica | Sin DevTools | Con DevTools | Overhead |
|---------|--------------|--------------|----------|
| **Frame time** | 16ms (60 FPS) | 18ms | +12% |
| **Memory** | 50 MB | 55 MB | +10% |

**Optimizaciones:**
- Lazy serialization (solo enviar cuando DevTools está abierto)
- Throttling de updates (max 30 FPS)
- Sampling profiling (no overhead si no está profiling)

---

### **10.2. Release Mode**

**En release builds:**
- ✅ Agent completamente removido (no overhead)
- ✅ `#[cfg(debug_assertions)]` asegura zero-cost en producción

---

## 11. Testing

### **11.1. Unit Tests**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_widget_tree_serialization() {
        let tree = WidgetTree::new();
        let json = tree.to_json();
        assert!(json.contains("\"type\":\"widget_tree\""));
    }
}
```

---

### **11.2. Integration Tests**

**Escenario:** Iniciar app → Abrir DevTools → Verificar widget tree.

```rust
#[tokio::test]
async fn test_devtools_connection() {
    let app = start_test_app().await;
    let devtools = DevToolsClient::connect("ws://localhost:9229").await.unwrap();
    
    let tree = devtools.get_widget_tree().await.unwrap();
    assert!(tree.root.widget_type == "MyApp");
}
```

---

## 12. Roadmap

### **Fase 1: MVP (Sprint 6-7)**
- ✅ UI Inspector (widget tree, properties panel)
- ✅ Basic signal graph (static view)
- ✅ WebSocket server

### **Fase 2: Advanced (Sprint 8-9)**
- ✅ Live editing
- ✅ Layout debugging
- ✅ Signal graph con timeline

### **Fase 3: Profiling (Sprint 10+)**
- ✅ CPU flamegraph
- ✅ Memory profiling
- ✅ Network inspector

---

## 13. Referencias

- **React DevTools**: https://github.com/facebook/react/tree/main/packages/react-devtools
- **Flutter DevTools**: https://docs.flutter.dev/tools/devtools
- **Chrome DevTools Protocol**: https://chromedevtools.github.io/devtools-protocol/
- **D3.js**: https://d3js.org/

---

**Autor:** Vela Core Team  
**Revisión:** 2025-11-30  
**Versión:** 1.0
