# 4. Diseño del Sistema de Signals y Reactividad

## 4.1 Arquitectura del Sistema Reactivo

### 4.1.1 Principios de Diseño

**Objetivos**:
1. **Reactividad fina**: Solo re-ejecutar lo mínimo necesario
2. **Performance**: Overhead mínimo en tracking y propagación
3. **Previsibilidad**: Orden de ejecución determinístico
4. **Debugging**: Rastreo claro de dependencias y cambios
5. **Memory efficiency**: Limpieza automática de dependencias muertas

**Modelo conceptual**:
```
Signal (fuente de verdad)
  ↓ (dependencia)
Computed (valor derivado)
  ↓ (dependencia)
Effect (efecto secundario)
  ↓
DOM/Side effects
```

---

### 4.1.2 Grafo de Dependencias Reactivo

**Representación**:
```rust
struct ReactiveGraph {
  // Nodes
  signals: HashMap<SignalId, SignalNode>,
  computeds: HashMap<ComputedId, ComputedNode>,
  effects: HashMap<EffectId, EffectNode>,
  
  // Edges (dependencias)
  dependencies: HashMap<NodeId, HashSet<NodeId>>,
  dependents: HashMap<NodeId, HashSet<NodeId>>,
  
  // Tracking context
  activeContext: Option<ReactiveContext>,
  contextStack: Vec<ReactiveContext>,
  
  // Batching
  batchDepth: Int,
  dirtyNodes: HashSet<NodeId>,
  
  // Memory management
  gcRoots: HashSet<NodeId>,
  refCounts: HashMap<NodeId, Int>,
}

type NodeId = union {
  Signal(SignalId),
  Computed(ComputedId),
  Effect(EffectId)
}
```

---

## 4.2 Signal<T> - Implementación

### 4.2.1 Estructura Interna

```rust
struct SignalNode<T> {
  id: SignalId,
  value: T,
  dependents: HashSet<NodeId>,
  version: u64,  // Para dirty checking optimizado
}

impl<T> SignalNode<T> {
  fn new(initial_value: T) -> Self {
    SignalNode {
      id: SignalId::new(),
      value: initial_value,
      dependents: HashSet::new(),
      version: 0,
    }
  }
  
  fn get(&self, graph: &mut ReactiveGraph) -> &T {
    // Track dependency
    if let Some(context) = graph.activeContext {
      graph.addDependency(context.nodeId, NodeId::Signal(self.id));
      self.dependents.insert(context.nodeId);
    }
    
    &self.value
  }
  
  fn set(&mut self, new_value: T, graph: &mut ReactiveGraph) {
    if self.value == new_value {
      return;  // No cambió, skip propagation
    }
    
    self.value = new_value;
    self.version += 1;
    
    // Mark dependents as dirty
    for dependent in &self.dependents {
      graph.markDirty(dependent);
    }
    
    // Schedule update
    if graph.batchDepth == 0 {
      graph.processBatch();
    }
  }
}
```

### 4.2.2 API Pública

```vela
// Usage example
let count = signal(0);

// Reading (automatic tracking)
effect(() => {
  print("Count is: ${count.value}");  // Tracked!
});

// Writing
state {
  count.value += 1;  // Triggers re-execution of effect
}
```

---

## 4.3 Computed<T> - Implementación

### 4.3.1 Estructura Interna

```rust
struct ComputedNode<T> {
  id: ComputedId,
  value: Option<T>,
  computeFn: Box<dyn Fn() -> T>,
  dependencies: HashSet<NodeId>,
  dependents: HashSet<NodeId>,
  dirty: bool,
  version: u64,
}

impl<T> ComputedNode<T> {
  fn new(compute_fn: impl Fn() -> T + 'static) -> Self {
    ComputedNode {
      id: ComputedId::new(),
      value: None,
      computeFn: Box::new(compute_fn),
      dependencies: HashSet::new(),
      dependents: HashSet::new(),
      dirty: true,
      version: 0,
    }
  }
  
  fn get(&mut self, graph: &mut ReactiveGraph) -> &T {
    // Track dependency
    if let Some(context) = graph.activeContext {
      graph.addDependency(context.nodeId, NodeId::Computed(self.id));
      self.dependents.insert(context.nodeId);
    }
    
    // Lazy evaluation
    if self.dirty {
      self.recompute(graph);
    }
    
    self.value.as_ref().unwrap()
  }
  
  fn recompute(&mut self, graph: &mut ReactiveGraph) {
    // Clear old dependencies
    for dep in &self.dependencies {
      graph.removeDependency(NodeId::Computed(self.id), *dep);
    }
    self.dependencies.clear();
    
    // Create tracking context
    let context = ReactiveContext {
      nodeId: NodeId::Computed(self.id),
    };
    graph.pushContext(context);
    
    // Execute compute function (tracks new dependencies)
    let new_value = (self.computeFn)();
    
    graph.popContext();
    
    // Update value
    self.value = Some(new_value);
    self.dirty = false;
    self.version += 1;
  }
  
  fn markDirty(&mut self, graph: &mut ReactiveGraph) {
    if self.dirty {
      return;  // Already dirty
    }
    
    self.dirty = true;
    
    // Propagate to dependents
    for dependent in &self.dependents {
      graph.markDirty(dependent);
    }
  }
}
```

### 4.3.2 Optimizaciones

**Memoization inteligente**:
```rust
// Si el valor computed no cambió después de recompute,
// no propagamos a dependientes
fn recompute(&mut self, graph: &mut ReactiveGraph) {
  let old_value = self.value.clone();
  
  // ... recompute logic ...
  
  let new_value = (self.computeFn)();
  
  if Some(&new_value) == old_value.as_ref() {
    // Valor no cambió, no propagar
    self.dirty = false;
    return;
  }
  
  self.value = Some(new_value);
  self.dirty = false;
  
  // Propagar a dependientes
  for dependent in &self.dependents {
    graph.markDirty(dependent);
  }
}
```

---

## 4.4 Effect - Implementación

### 4.4.1 Estructura Interna

```rust
struct EffectNode {
  id: EffectId,
  effectFn: Box<dyn Fn()>,
  dependencies: HashSet<NodeId>,
  active: bool,
  scheduled: bool,
}

impl EffectNode {
  fn new(effect_fn: impl Fn() + 'static) -> Self {
    EffectNode {
      id: EffectId::new(),
      effectFn: Box::new(effect_fn),
      dependencies: HashSet::new(),
      active: true,
      scheduled: false,
    }
  }
  
  fn run(&mut self, graph: &mut ReactiveGraph) {
    if !self.active {
      return;
    }
    
    // Clear old dependencies
    for dep in &self.dependencies {
      graph.removeDependency(NodeId::Effect(self.id), *dep);
    }
    self.dependencies.clear();
    
    // Create tracking context
    let context = ReactiveContext {
      nodeId: NodeId::Effect(self.id),
    };
    graph.pushContext(context);
    
    // Execute effect (tracks dependencies)
    (self.effectFn)();
    
    graph.popContext();
    
    self.scheduled = false;
  }
  
  fn schedule(&mut self) {
    if self.scheduled {
      return;
    }
    self.scheduled = true;
  }
}
```

### 4.4.2 Cleanup

```vela
// Effect with cleanup
let cleanup = effect(() => {
  let timer = setInterval(() => {
    print("Tick");
  }, 1000);
  
  // Return cleanup function
  return () => {
    clearInterval(timer);
  };
});

// Later: cleanup.stop() ejecuta la función de limpieza
```

---

## 4.5 Scheduler Reactivo

### 4.5.1 Algoritmo de Propagación

```rust
impl ReactiveGraph {
  fn processBatch(&mut self) {
    // 1. Topological sort of dirty nodes
    let sorted_nodes = self.topologicalSort(self.dirtyNodes.clone());
    
    // 2. Process computed nodes first
    for node_id in sorted_nodes {
      match node_id {
        NodeId::Computed(id) => {
          let computed = self.computeds.get_mut(&id).unwrap();
          if computed.dirty {
            computed.recompute(self);
          }
        },
        _ => {}
      }
    }
    
    // 3. Process effects last
    for node_id in sorted_nodes {
      match node_id {
        NodeId::Effect(id) => {
          let effect = self.effects.get_mut(&id).unwrap();
          if effect.scheduled {
            effect.run(self);
          }
        },
        _ => {}
      }
    }
    
    // 4. Clear dirty set
    self.dirtyNodes.clear();
  }
  
  fn topologicalSort(&self, nodes: HashSet<NodeId>) -> Vec<NodeId> {
    let mut sorted = Vec::new();
    let mut visited = HashSet::new();
    let mut temp_mark = HashSet::new();
    
    for node in nodes {
      if !visited.contains(&node) {
        self.visit(node, &mut visited, &mut temp_mark, &mut sorted);
      }
    }
    
    sorted
  }
  
  fn visit(
    &self,
    node: NodeId,
    visited: &mut HashSet<NodeId>,
    temp_mark: &mut HashSet<NodeId>,
    sorted: &mut Vec<NodeId>
  ) {
    if visited.contains(&node) {
      return;
    }
    
    if temp_mark.contains(&node) {
      panic!("Circular dependency detected!");
    }
    
    temp_mark.insert(node);
    
    // Visit dependencies first
    if let Some(deps) = self.dependencies.get(&node) {
      for dep in deps {
        self.visit(*dep, visited, temp_mark, sorted);
      }
    }
    
    temp_mark.remove(&node);
    visited.insert(node);
    sorted.push(node);
  }
}
```

### 4.5.2 Batching

```vela
// Sin batching: 3 updates
count.value = 1;  // Effect runs
count.value = 2;  // Effect runs
count.value = 3;  // Effect runs

// Con batching: 1 update
batch(() => {
  count.value = 1;
  count.value = 2;
  count.value = 3;
  // Effect runs solo 1 vez al final
});
```

**Implementación**:
```rust
impl ReactiveGraph {
  fn startBatch(&mut self) {
    self.batchDepth += 1;
  }
  
  fn endBatch(&mut self) {
    self.batchDepth -= 1;
    
    if self.batchDepth == 0 {
      self.processBatch();
    }
  }
}

fn batch(fn: impl FnOnce()) {
  GRAPH.startBatch();
  fn();
  GRAPH.endBatch();
}
```

---

## 4.6 Watch - Implementación

```rust
struct WatchNode<T> {
  source: Signal<T>,
  callback: Box<dyn Fn(T, T)>,  // (new, old)
  oldValue: Option<T>,
  immediate: bool,
}

impl<T: Clone> WatchNode<T> {
  fn create(
    source: Signal<T>,
    callback: impl Fn(T, T) + 'static,
    immediate: bool
  ) -> Self {
    let old_value = if immediate {
      None
    } else {
      Some(source.value.clone())
    };
    
    let node = WatchNode {
      source,
      callback: Box::new(callback),
      oldValue: old_value,
      immediate,
    };
    
    // Create effect to watch changes
    effect(move || {
      let new_value = source.value.clone();
      
      if let Some(old) = node.oldValue.as_ref() {
        if new_value != *old {
          (node.callback)(new_value.clone(), old.clone());
        }
      } else if node.immediate {
        // First run with immediate=true
        (node.callback)(new_value.clone(), new_value.clone());
      }
      
      node.oldValue = Some(new_value);
    });
    
    node
  }
}
```

---

## 4.7 Integración con UI

### 4.7.1 Reconciliación Reactiva

```vela
fn Counter(): Widget {
  let count = signal(0);
  
  // Esta función se re-ejecuta cuando count cambia
  return Container {
    Text("Count: ${count.value}"),  // Tracking automático
    Button {
      label: "Increment",
      onClick: () => state { count.value += 1; }
    }
  };
}
```

**Bajo el capó**:
```rust
struct WidgetNode {
  buildFn: Box<dyn Fn() -> VNode>,
  vnode: VNode,
  effect: EffectNode,
}

impl WidgetNode {
  fn create(build_fn: impl Fn() -> VNode + 'static) -> Self {
    let mut node = WidgetNode {
      buildFn: Box::new(build_fn),
      vnode: VNode::empty(),
      effect: EffectNode::new(|| {}),
    };
    
    // Create effect that re-builds on signal changes
    node.effect = effect(move || {
      let new_vnode = (node.buildFn)();
      
      // Reconcile with old vnode
      let patches = diff(&node.vnode, &new_vnode);
      applyPatches(patches);
      
      node.vnode = new_vnode;
    });
    
    node
  }
}
```

### 4.7.2 Virtual DOM y Diffing

```rust
enum VNode {
  Element {
    tag: String,
    props: HashMap<String, Value>,
    children: Vec<VNode>,
  },
  Text(String),
  Component {
    componentFn: Box<dyn Fn() -> VNode>,
  },
}

enum Patch {
  ReplaceNode { path: Vec<usize>, newNode: VNode },
  UpdateProps { path: Vec<usize>, props: HashMap<String, Value> },
  InsertChild { path: Vec<usize>, index: usize, node: VNode },
  RemoveChild { path: Vec<usize>, index: usize },
}

fn diff(old: &VNode, new: &VNode) -> Vec<Patch> {
  // Simple diffing algorithm
  match (old, new) {
    (VNode::Text(old_text), VNode::Text(new_text)) => {
      if old_text != new_text {
        vec![Patch::ReplaceNode { path: vec![], newNode: new.clone() }]
      } else {
        vec![]
      }
    },
    
    (VNode::Element { tag: old_tag, props: old_props, children: old_children },
     VNode::Element { tag: new_tag, props: new_props, children: new_children }) => {
      let mut patches = vec![];
      
      if old_tag != new_tag {
        patches.push(Patch::ReplaceNode { path: vec![], newNode: new.clone() });
        return patches;
      }
      
      // Diff props
      if old_props != new_props {
        patches.push(Patch::UpdateProps { path: vec![], props: new_props.clone() });
      }
      
      // Diff children (simplified)
      for i in 0..max(old_children.len(), new_children.len()) {
        // ... recursive diffing
      }
      
      patches
    },
    
    _ => vec![Patch::ReplaceNode { path: vec![], newNode: new.clone() }]
  }
}
```

---

## 4.8 Memory Management

### 4.8.1 Garbage Collection de Signals

**Problema**: Signals no usados deben ser limpiados

**Solución**: Reference counting con detección de ciclos

```rust
impl ReactiveGraph {
  fn gc(&mut self) {
    // 1. Mark phase: mark all reachable nodes from roots
    let mut reachable = HashSet::new();
    for root in &self.gcRoots {
      self.mark(*root, &mut reachable);
    }
    
    // 2. Sweep phase: remove unreachable nodes
    self.signals.retain(|id, _| reachable.contains(&NodeId::Signal(*id)));
    self.computeds.retain(|id, _| reachable.contains(&NodeId::Computed(*id)));
    self.effects.retain(|id, _| reachable.contains(&NodeId::Effect(*id)));
  }
  
  fn mark(&self, node: NodeId, reachable: &mut HashSet<NodeId>) {
    if reachable.contains(&node) {
      return;
    }
    
    reachable.insert(node);
    
    // Mark dependents
    if let Some(deps) = self.dependents.get(&node) {
      for dep in deps {
        self.mark(*dep, reachable);
      }
    }
  }
}
```

### 4.8.2 Automatic Cleanup

```vela
fn Component(): Widget {
  let signal = signal(0);
  let effect = effect(() => {
    print(signal.value);
  });
  
  // Cuando Component se destruye:
  // 1. effect se detiene automáticamente
  // 2. Si signal no tiene otros dependientes, se libera
  // 3. Dependencias se limpian del grafo
}
```

---

## 4.9 Debugging y DevTools

### 4.9.1 Signal Inspector

```rust
struct SignalDebugInfo {
  id: SignalId,
  name: String,
  value: String,  // Serialized
  dependents: Vec<String>,
  dependencies: Vec<String>,
  updateCount: u64,
  lastUpdated: Timestamp,
}

impl ReactiveGraph {
  fn debugInfo(&self) -> Vec<SignalDebugInfo> {
    // Serialize graph state for DevTools
    self.signals.iter().map(|(id, node)| {
      SignalDebugInfo {
        id: *id,
        name: self.getDebugName(*id),
        value: format!("{:?}", node.value),
        dependents: self.getDependentNames(*id),
        dependencies: vec![],
        updateCount: node.version,
        lastUpdated: Timestamp::now(),
      }
    }).collect()
  }
  
  fn visualizeGraph(&self) -> String {
    // Generate DOT format for graph visualization
    // ...
  }
}
```

### 4.9.2 Performance Profiling

```rust
struct ReactiveStats {
  totalSignals: usize,
  totalComputed: usize,
  totalEffects: usize,
  batchesProcessed: u64,
  averageBatchSize: f64,
  recomputations: u64,
  effectExecutions: u64,
}

impl ReactiveGraph {
  fn stats(&self) -> ReactiveStats {
    // Collect performance metrics
    // ...
  }
}
```

---

## 4.10 Advanced Patterns

### 4.10.1 Derived State (Multiple Signals)

```vela
let firstName = signal("John");
let lastName = signal("Doe");

let fullName = computed(() => "${firstName.value} ${lastName.value}");

// fullName se actualiza cuando firstName O lastName cambian
```

### 4.10.2 Async Signals

```vela
let userId = signal(1);

let userData = computed(async () => {
  let response = await fetch("/api/users/${userId.value}");
  return await response.json();
});

// userData es Promise-based computed
effect(() => {
  userData.value.then((data) => {
    print("User: ${data.name}");
  });
});
```

### 4.10.3 Signal Arrays

```vela
let todos = signal<List<Todo>>([]);

// Computed basado en lista
let completedCount = computed(() => {
  return todos.value.filter((todo) => todo.completed).length;
});

// Optimización: track individual items
let todoSignals = todos.value.map((todo) => signal(todo));
```

---

**FIN DEL DOCUMENTO: Sistema de Signals y Reactividad**

Este documento cubre la implementación completa del sistema reactivo de Vela.
