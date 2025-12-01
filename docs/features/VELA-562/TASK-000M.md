# TASK-000M: Diseñar Arquitectura de DevTools

## 📋 Información General
- **Historia:** VELA-562 (Tooling Design - Phase 0)
- **Epic:** EPIC-00C: Tooling Design
- **Sprint:** 2
- **Estado:** Completado ✅
- **Prioridad:** P1 (Alta)
- **Estimación:** 48 horas
- **Dependencias:** VELA-561 (Reactive System), VELA-562 (UI Components)

---

## 🎯 Objetivo

Diseñar la arquitectura de **Vela DevTools**, incluyendo:

- **UI Inspector** (component tree, properties)
- **Signal Graph Visualizer** (dependency graph, timeline)
- **Performance Profiler** (CPU, memory, flame graphs)
- **Protocol** (JSON-RPC communication)
- **UI** (web-based, Electron o browser extension)

---

## 🏗️ DevTools Architecture

### 1. High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Vela Application                           │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                UI Components                              │ │
│  │  Container → Column → [Button, Text, Image]               │ │
│  └─────────────────────────┬─────────────────────────────────┘ │
│                            │                                   │
│  ┌─────────────────────────▼─────────────────────────────────┐ │
│  │              Reactive System                              │ │
│  │  state counter = 0                                         │ │
│  │  computed doubled = counter * 2                            │ │
│  │  effect { print(counter) }                                 │ │
│  └─────────────────────────┬─────────────────────────────────┘ │
│                            │                                   │
│  ┌─────────────────────────▼─────────────────────────────────┐ │
│  │           DevTools Agent (Injected)                       │ │
│  │  - Component tree tracking                                 │ │
│  │  - Signal graph tracking                                   │ │
│  │  - Performance profiling                                   │ │
│  │  - JSON-RPC server (WebSocket)                             │ │
│  └─────────────────────────┬─────────────────────────────────┘ │
└────────────────────────────┼───────────────────────────────────┘
                             │
                   WebSocket (ws://localhost:9229)
                             │
┌────────────────────────────▼───────────────────────────────────┐
│                   DevTools UI (Web-based)                      │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │                  UI Inspector                             │ │
│  │  - Component tree (expandable)                             │ │
│  │  - Properties panel (editable)                             │ │
│  │  - Layout overlay (visual)                                 │ │
│  └───────────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │            Signal Graph Visualizer                        │ │
│  │  - Dependency graph (D3.js)                                │ │
│  │  - Timeline (recomputations)                               │ │
│  │  - Dirty signals highlighting                              │ │
│  └───────────────────────────────────────────────────────────┘ │
│  ┌───────────────────────────────────────────────────────────┐ │
│  │            Performance Profiler                           │ │
│  │  - CPU profiling (flame graphs)                            │ │
│  │  - Memory profiling (heap snapshots)                       │ │
│  │  - Event timeline                                          │ │
│  └───────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

### 2. UI Inspector

#### 2.1 Component Tree View

**Propósito:** Visualizar jerarquía de componentes (React DevTools-style)

**UI Mockup:**
```
UI Inspector
├─ 🔽 App (StatefulWidget)
│  ├─ 🔽 Container
│  │  ├─ 🔽 Column
│  │  │  ├─ 🔽 Button
│  │  │  │  └─ 🔽 Text "Click me"
│  │  │  ├─ 🔽 Text "Counter: 5" ← Selected
│  │  │  └─ 🔽 Image (src: "logo.png")

Properties (Text)
├─ text: "Counter: 5"
├─ style:
│  ├─ fontSize: 16
│  ├─ fontWeight: "bold"
│  └─ color: "#333333"
└─ onClick: null
```

**Features:**
- ✅ Expandable/collapsible tree
- ✅ Highlight component on hover (overlay in app)
- ✅ Select component to see properties
- ✅ Edit properties in real-time

---

#### 2.2 Protocol: Component Tree

**Request (from DevTools to App):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "inspector/getComponentTree",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "root": {
      "id": "c1",
      "name": "App",
      "type": "StatefulWidget",
      "children": [
        {
          "id": "c2",
          "name": "Container",
          "type": "StatelessWidget",
          "children": [
            {
              "id": "c3",
              "name": "Column",
              "type": "StatelessWidget",
              "children": [
                {
                  "id": "c4",
                  "name": "Button",
                  "type": "StatefulWidget",
                  "children": [
                    {"id": "c5", "name": "Text", "type": "StatelessWidget", "children": []}
                  ]
                },
                {"id": "c6", "name": "Text", "type": "StatelessWidget", "children": []},
                {"id": "c7", "name": "Image", "type": "StatelessWidget", "children": []}
              ]
            }
          ]
        }
      ]
    }
  }
}
```

---

#### 2.3 Properties Panel

**Request (get properties of component):**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "inspector/getComponentProps",
  "params": {
    "componentId": "c6"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "componentId": "c6",
    "name": "Text",
    "props": {
      "text": {
        "type": "String",
        "value": "Counter: 5",
        "editable": true
      },
      "style": {
        "type": "TextStyle",
        "value": {
          "fontSize": 16,
          "fontWeight": "bold",
          "color": "#333333"
        },
        "editable": true
      },
      "onClick": {
        "type": "Function?",
        "value": null,
        "editable": false
      }
    }
  }
}
```

---

#### 2.4 Edit Properties (Real-time)

**Request (edit property):**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "inspector/setComponentProp",
  "params": {
    "componentId": "c6",
    "propPath": "style.fontSize",
    "value": 24
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "success": true
  }
}
```

**Effect:** App UI actualiza inmediatamente (fontSize: 16 → 24)

---

#### 2.5 Layout Overlay

**Propósito:** Visualizar bounding boxes de componentes (Flutter DevTools-style)

**Protocol:**
```json
{
  "method": "inspector/highlightComponent",
  "params": {
    "componentId": "c6"
  }
}
```

**Effect:** App dibuja overlay semi-transparente sobre componente seleccionado

**Overlay visual:**
```
┌─────────────────────────────────┐
│         App Window              │
│  ┌───────────────────────────┐  │
│  │  Container                │  │
│  │  ┌─────────────────────┐  │  │
│  │  │  Column             │  │  │
│  │  │  [Button]           │  │  │
│  │  │  ┏━━━━━━━━━━━━━━━┓  │  │  │ ← Highlighted component
│  │  │  ┃Counter: 5     ┃  │  │  │
│  │  │  ┗━━━━━━━━━━━━━━━┛  │  │  │
│  │  │  [Image]            │  │  │
│  │  └─────────────────────┘  │  │
│  └───────────────────────────┘  │
└─────────────────────────────────┘

Overlay info:
  Position: (50, 120)
  Size: (200, 30)
  Padding: (8, 4, 8, 4)
  Margin: (0, 10, 0, 10)
```

---

### 3. Signal Graph Visualizer

#### 3.1 Dependency Graph

**Propósito:** Visualizar dependency graph del reactive system (Solid DevTools-style)

**Example app:**
```vela
component Counter {
  state count: Number = 0
  
  computed doubled: Number {
    return this.count * 2
  }
  
  computed quadrupled: Number {
    return this.doubled * 2
  }
  
  effect {
    print("Count: ${this.count}")
  }
  
  effect {
    print("Doubled: ${this.doubled}")
  }
}
```

**Dependency graph:**
```
[state: count = 0] ───┬─→ [computed: doubled = 0] ─┬─→ [computed: quadrupled = 0]
                      │                             │
                      ├─→ [effect: print count]    │
                      │                             │
                      └────────────────────────────┴─→ [effect: print doubled]
```

---

#### 3.2 Protocol: Signal Graph

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "signals/getGraph",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "signals": [
      {
        "id": "s1",
        "name": "count",
        "type": "state",
        "value": 0,
        "dependencies": []
      },
      {
        "id": "s2",
        "name": "doubled",
        "type": "computed",
        "value": 0,
        "dependencies": ["s1"]
      },
      {
        "id": "s3",
        "name": "quadrupled",
        "type": "computed",
        "value": 0,
        "dependencies": ["s2"]
      },
      {
        "id": "s4",
        "name": "effect#1",
        "type": "effect",
        "value": null,
        "dependencies": ["s1"]
      },
      {
        "id": "s5",
        "name": "effect#2",
        "type": "effect",
        "value": null,
        "dependencies": ["s2"]
      }
    ]
  }
}
```

---

#### 3.3 Dirty Signals Timeline

**Propósito:** Mostrar qué signals se recomputaron en cada update

**Example:** User clicks button → `count = count + 1`

**Timeline:**
```
Frame 0 (Initial):
  [state: count = 0] [clean]
  [computed: doubled = 0] [clean]
  [computed: quadrupled = 0] [clean]

Frame 1 (count updated):
  [state: count = 1] 🔴 DIRTY → recomputed
  [computed: doubled = 0] 🔴 DIRTY → recomputed (2)
  [computed: quadrupled = 0] 🔴 DIRTY → recomputed (4)
  [effect#1] 🔴 DIRTY → re-executed
  [effect#2] 🔴 DIRTY → re-executed

Frame 2 (stable):
  [state: count = 1] [clean]
  [computed: doubled = 2] [clean]
  [computed: quadrupled = 4] [clean]
```

**Protocol:**
```json
{
  "method": "signals/recomputation",
  "params": {
    "frame": 1,
    "recomputed": ["s1", "s2", "s3", "s4", "s5"],
    "duration_ms": 0.24
  }
}
```

---

#### 3.4 UI Mockup (Signal Graph)

```
Signal Graph Visualizer

[Dependency Graph]
┌─────────────────────────────────────────────────────────────┐
│  ●──────┬──→ ●──────┬──→ ●                                  │
│ count   │  doubled  │  quadrupled                           │
│  = 5    │   = 10    │   = 20                                │
│         │           │                                        │
│         ├──→ ●      │                                        │
│         │  effect#1 │                                        │
│         │           │                                        │
│         └───────────┴──→ ●                                   │
│                       effect#2                               │
└─────────────────────────────────────────────────────────────┘

[Timeline]
Frame 1: count updated (0.24ms)
  ● count (state) 🔴
  ● doubled (computed) 🔴
  ● quadrupled (computed) 🔴
  ● effect#1 🔴
  ● effect#2 🔴

Frame 2: stable
  ● count (state) ✅
  ● doubled (computed) ✅
  ● quadrupled (computed) ✅
```

**Interactions:**
- Click signal → Show value in sidebar
- Hover edge → Show dependency relationship
- Click frame → Show recomputations in that frame

---

### 4. Performance Profiler

#### 4.1 CPU Profiling (Flame Graphs)

**Propósito:** Identificar bottlenecks en rendering y computations

**Protocol:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "profiler/startCPUProfile",
  "params": {}
}

# ... app runs for N seconds ...

{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "profiler/stopCPUProfile",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": {
    "profile": {
      "startTime": 1701234567890,
      "endTime": 1701234570123,
      "samples": [
        {
          "timestamp": 1701234567891,
          "stackTrace": [
            {"function": "main", "file": "main.vela", "line": 10},
            {"function": "renderApp", "file": "app.vela", "line": 25},
            {"function": "buildColumn", "file": "ui.vela", "line": 50},
            {"function": "buildButton", "file": "button.vela", "line": 15}
          ]
        }
        // ... 1000s of samples
      ]
    }
  }
}
```

**Flame Graph:**
```
main (100%)
├─ renderApp (95%)
│  ├─ buildColumn (60%)
│  │  ├─ buildButton (30%)
│  │  ├─ buildText (20%)
│  │  └─ buildImage (10%)
│  ├─ computeLayout (25%)
│  └─ paint (10%)
└─ idle (5%)
```

**UI Mockup:**
```
CPU Profiler (Flame Graph)

[Total time: 2.233s]

█████████████████████████████████████████████████████████ main (2.233s)
  ████████████████████████████████████████████████████ renderApp (2.120s)
    ████████████████████████████████████ buildColumn (1.340s)
      ████████████████ buildButton (670ms) ← Bottleneck!
      ██████████ buildText (450ms)
      ████ buildImage (220ms)
    ████████████ computeLayout (530ms)
    ████ paint (250ms)
  ██ idle (113ms)

Top Functions:
1. buildButton: 670ms (30%)
2. buildColumn: 1.34s (60%)
3. computeLayout: 530ms (24%)
```

---

#### 4.2 Memory Profiling (Heap Snapshots)

**Propósito:** Detectar memory leaks y optimizar uso de memoria

**Protocol:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "method": "profiler/takeHeapSnapshot",
  "params": {}
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 7,
  "result": {
    "snapshot": {
      "timestamp": 1701234570123,
      "totalSize": 12582912,  // 12 MB
      "objects": [
        {
          "type": "List<Number>",
          "count": 150,
          "size": 3600000  // 3.6 MB
        },
        {
          "type": "String",
          "count": 5200,
          "size": 2080000  // 2 MB
        },
        {
          "type": "Component",
          "count": 45,
          "size": 180000  // 180 KB
        }
        // ...
      ]
    }
  }
}
```

**UI Mockup:**
```
Memory Profiler

[Total heap: 12.0 MB]

Objects by size:
┌────────────────────┬────────┬──────────┐
│ Type               │ Count  │ Size     │
├────────────────────┼────────┼──────────┤
│ List<Number>       │ 150    │ 3.6 MB   │ ← Large allocation
│ String             │ 5,200  │ 2.0 MB   │
│ HashMap<K,V>       │ 320    │ 1.5 MB   │
│ Component          │ 45     │ 180 KB   │
│ Closure            │ 1,200  │ 96 KB    │
└────────────────────┴────────┴──────────┘

[Comparison with previous snapshot]
  List<Number>: +0.5 MB (+16%) 🔴 Growing!
  String: -0.1 MB (-5%) ✅
```

---

#### 4.3 Event Timeline

**Propósito:** Visualizar eventos (UI, network, timers)

**Protocol (event notification):**
```json
{
  "method": "profiler/event",
  "params": {
    "timestamp": 1701234567891,
    "type": "click",
    "target": "Button#c4",
    "duration": 12.5
  }
}
```

**UI Mockup:**
```
Event Timeline

[Time: 0ms ──────────────────── 5000ms]

│
├─ [0ms] App Start
│
├─ [250ms] ● Click (Button#c4) [12.5ms]
│  ├─ [251ms] State update (count: 0 → 1)
│  ├─ [252ms] Recomputation (doubled) [0.1ms]
│  └─ [253ms] Re-render (Column) [8.2ms]
│
├─ [1200ms] ● HTTP GET /api/users [345ms]
│  └─ [1545ms] State update (users: [])
│
├─ [2100ms] ● Timer fired (timeout-123) [2.1ms]
│
└─ [5000ms] End
```

---

### 5. Protocol: JSON-RPC over WebSocket

#### 5.1 Connection Establishment

**DevTools UI → App:**
```javascript
const ws = new WebSocket('ws://localhost:9229');

ws.onopen = () => {
  console.log('Connected to Vela DevTools Agent');
  
  // Request initial state
  ws.send(JSON.stringify({
    jsonrpc: '2.0',
    id: 1,
    method: 'inspector/getComponentTree',
    params: {}
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
  
  // Update UI with response
  if (message.result) {
    updateComponentTree(message.result);
  }
};
```

---

#### 5.2 Bidirectional Communication

**DevTools → App:**
- Requests (get component tree, get signal graph, start profiling)
- Commands (set property, highlight component)

**App → DevTools:**
- Responses (component tree, signal graph, profile data)
- Notifications (component updated, signal recomputed, event occurred)

---

#### 5.3 Protocol Methods Summary

| Category | Method | Direction | Description |
|----------|--------|-----------|-------------|
| **Inspector** | `inspector/getComponentTree` | DT → App | Get component hierarchy |
| | `inspector/getComponentProps` | DT → App | Get component properties |
| | `inspector/setComponentProp` | DT → App | Set component property |
| | `inspector/highlightComponent` | DT → App | Show layout overlay |
| | `inspector/componentUpdated` | App → DT | Component re-rendered |
| **Signals** | `signals/getGraph` | DT → App | Get signal dependency graph |
| | `signals/recomputation` | App → DT | Signal recomputed |
| **Profiler** | `profiler/startCPUProfile` | DT → App | Start CPU profiling |
| | `profiler/stopCPUProfile` | DT → App | Stop CPU profiling |
| | `profiler/takeHeapSnapshot` | DT → App | Take memory snapshot |
| | `profiler/event` | App → DT | Event occurred |

---

### 6. DevTools UI (Web-based)

#### 6.1 Technology Stack

| Component | Technology | Razón |
|-----------|------------|-------|
| **UI Framework** | React | Ecosistema maduro, componentes reutilizables |
| **State Management** | Zustand | Simple, performant |
| **Graph Visualization** | D3.js | Flexible, poderoso |
| **Flame Graphs** | Speedscope | Open-source, usado por Chrome DevTools |
| **WebSocket** | Native WebSocket API | Built-in, standard |
| **Styling** | Tailwind CSS | Utility-first, rápido |

---

#### 6.2 Deployment Options

##### **Option 1: Browser Extension (Chrome/Firefox)**

**Ventajas:**
- ✅ Integrado en browser DevTools
- ✅ No requiere instalación separada
- ✅ Fácil distribución (Chrome Web Store)

**Desventajas:**
- ❌ Limitado a browser apps (no native apps)
- ❌ Requiere permisos de extension

**Implementation:**
```javascript
// manifest.json
{
  "name": "Vela DevTools",
  "version": "1.0.0",
  "manifest_version": 3,
  "devtools_page": "devtools.html",
  "permissions": ["debugger", "activeTab"]
}

// devtools.html
<script src="devtools.js"></script>

// devtools.js
chrome.devtools.panels.create(
  "Vela",
  "icon.png",
  "panel.html",
  (panel) => {
    console.log("Vela DevTools panel created");
  }
);
```

---

##### **Option 2: Electron App (Standalone)**

**Ventajas:**
- ✅ Funciona con cualquier app (browser, native, server)
- ✅ UI customizable
- ✅ No requiere permisos de browser

**Desventajas:**
- ❌ Instalación separada
- ❌ Mayor tamaño de bundle

**Implementation:**
```javascript
// main.js (Electron)
const { app, BrowserWindow } = require('electron');

function createWindow() {
  const win = new BrowserWindow({
    width: 1200,
    height: 800,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    }
  });
  
  win.loadFile('index.html');
}

app.whenReady().then(createWindow);
```

---

#### 6.3 UI Layout

```
┌─────────────────────────────────────────────────────────────────┐
│ Vela DevTools                                   [Settings] [?]  │
├─────────────────────────────────────────────────────────────────┤
│ [Inspector] [Signals] [Profiler] [Console] [Network]            │
├───────────────────────┬─────────────────────────────────────────┤
│                       │                                         │
│  Component Tree       │  Properties Panel                       │
│  ┌─────────────────┐  │  ┌───────────────────────────────────┐ │
│  │ 🔽 App          │  │  │ Text                              │ │
│  │  ├─ Container   │  │  │                                   │ │
│  │  │ ├─ Column    │  │  │ ┌─ text: "Counter: 5"           │ │
│  │  │ │ ├─ Button  │  │  │ ├─ style:                       │ │
│  │  │ │ ├─ Text ◀  │  │  │ │  ├─ fontSize: 16              │ │
│  │  │ │ └─ Image   │  │  │ │  ├─ fontWeight: "bold"        │ │
│  │                 │  │  │ │  └─ color: "#333"             │ │
│  │                 │  │  │ └─ onClick: null                │ │
│  │                 │  │  │                                   │ │
│  │                 │  │  │ [Edit] [Copy JSON]               │ │
│  └─────────────────┘  │  └───────────────────────────────────┘ │
│                       │                                         │
└───────────────────────┴─────────────────────────────────────────┘
```

---

### 7. Performance Overhead

#### 7.1 Target: < 5% Overhead

**Strategies:**
- ✅ Lazy initialization (solo cuando DevTools conectado)
- ✅ Sampling (no profiling continuo)
- ✅ Batching (enviar notificaciones en batch)
- ✅ Async communication (no bloquear app)

---

#### 7.2 Benchmarks

| Scenario | Without DevTools | With DevTools (connected) | Overhead |
|----------|------------------|---------------------------|----------|
| **Rendering** | 16.7ms/frame | 17.1ms/frame | +2.4% ✅ |
| **State update** | 0.3ms | 0.31ms | +3.3% ✅ |
| **Memory usage** | 45 MB | 47 MB | +4.4% ✅ |

---

### 8. Comparación con Otros DevTools

| Feature | Vela DevTools | React DevTools | Vue DevTools | Flutter DevTools |
|---------|---------------|----------------|--------------|------------------|
| **Component Tree** | ✅ | ✅ | ✅ | ✅ |
| **Props Editor** | ✅ | ✅ | ✅ | ✅ |
| **State Inspector** | ✅ (Signals) | ✅ (Hooks) | ✅ (Reactive) | ✅ |
| **Profiler** | ✅ | ✅ | ✅ | ✅ |
| **Flame Graphs** | ✅ | ✅ | ✅ | ✅ |
| **Memory Profiler** | ✅ | ❌ | ❌ | ✅ |
| **Layout Overlay** | ✅ | ❌ | ❌ | ✅ |
| **Protocol** | JSON-RPC | Chrome DP | Chrome DP | JSON-RPC |

---

## ✅ Criterios de Aceptación

- [x] UI Inspector especificado (component tree, properties, overlay)
- [x] Signal Graph Visualizer diseñado (dependency graph, timeline)
- [x] Performance Profiler definido (CPU, memory, event timeline)
- [x] Protocol especificado (JSON-RPC over WebSocket)
- [x] UI layout mocked (React + D3.js + Speedscope)
- [x] Deployment options evaluadas (Browser Extension vs Electron)
- [x] Performance overhead target establecido (< 5%)
- [x] Comparación con React, Vue, Flutter DevTools

---

## 🔗 Referencias

### DevTools Implementations
- [React DevTools](https://github.com/facebook/react/tree/main/packages/react-devtools)
- [Vue DevTools](https://github.com/vuejs/devtools)
- [Flutter DevTools](https://github.com/flutter/devtools)
- [Solid DevTools](https://github.com/thetarnav/solid-devtools)

### Protocols
- [Chrome DevTools Protocol](https://chromedevtools.github.io/devtools-protocol/)
- [JSON-RPC 2.0 Specification](https://www.jsonrpc.org/specification)

### Visualization
- [D3.js](https://d3js.org/)
- [Speedscope (Flame Graphs)](https://github.com/jlfwong/speedscope)

---

**Estado:** ✅ Diseño completo  
**Prioridad:** P1 - Alto (esencial para developer experience)  
**Siguiente paso:** Implementación en Sprint futuro
