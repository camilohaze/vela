# TASK-054: Implementar Widget Base Class

## 📋 Información General
- **Historia:** VELA-575 (Sprint 20 - UI Framework)
- **Estado:** ✅ Completada
- **Fecha:** 2025-12-05
- **Prioridad:** P0
- **Estimación:** 24 horas
- **Depende de:** TASK-053

## 🎯 Objetivo

Implementar las clases base del sistema de widgets de Vela:
- Widget (clase abstracta)
- StatelessWidget (widgets sin estado)
- StatefulWidget (widgets con estado)
- Key system (identificación única)
- BuildContext (acceso al árbol)
- Element (estado mutable interno)

## 🔨 Implementación

### 1. Key System (3 tipos de Keys)

```vela
# ValueKey: Key basada en valor primitivo
key: ValueKey<String> = ValueKey("user-123")

# ObjectKey: Key basada en identidad de objeto
key: ObjectKey = ObjectKey(myObject)

# GlobalKey: Key única globalmente
key: GlobalKey = GlobalKey()
```

**Uso principal**: Identificar widgets en listas dinámicas para reconciliación eficiente.

### 2. Widget Base Class

```vela
abstract class Widget {
  key: Option<Key> = None
  
  # Método principal - construir la UI
  abstract fn build(context: BuildContext) -> Widget
  
  # Lifecycle hooks
  mount() -> void { }
  beforeUpdate() -> void { }
  afterUpdate() -> void { }
  destroy() -> void { }
  
  # Comparación para reconciliación
  fn canUpdate(other: Widget) -> Bool {
    return type(this) == type(other) && this.key == other.key
  }
}
```

**Características:**
- ✅ Abstracta (no instanciable directamente)
- ✅ Inmutable (se recrea en cada rebuild)
- ✅ Ligera (sin estado mutable)
- ✅ 4 lifecycle hooks

### 3. StatelessWidget

```vela
widget Label extends StatelessWidget {
  text: String
  fontSize: Number = 14
  
  fn build(context: BuildContext) -> Widget {
    return Text(
      this.text,
      style: TextStyle(fontSize: this.fontSize)
    )
  }
}
```

**Características:**
- ✅ Solo props inmutables
- ✅ No tiene `state` variables
- ✅ build() llamado cuando props cambian
- ✅ Más eficiente que StatefulWidget
- ✅ Preferir cuando sea posible (99% de casos)

### 4. StatefulWidget

```vela
widget Counter extends StatefulWidget {
  state count: Number = 0
  
  fn build(context: BuildContext) -> Widget {
    return Column(
      children: [
        Text("Count: ${this.count}"),
        Button(
          text: "+1",
          onClick: () => this.count += 1  # Muta state → rebuild
        )
      ]
    )
  }
  
  # Optional lifecycle hooks
  mount() -> void {
    # Fetch initial data
  }
  
  destroy() -> void {
    # Cleanup
  }
}
```

**Características:**
- ✅ Puede tener `state` variables mutables
- ✅ Mutar state triggea rebuild automático
- ✅ Sistema reactivo integrado (usa Signals internamente)
- ✅ build() llamado cada vez que state cambia
- ✅ Lifecycle completo (mount, update, destroy)

### 5. BuildContext

```vela
widget MyWidget extends StatelessWidget {
  fn build(context: BuildContext) -> Widget {
    # Acceder al theme
    theme: Theme = context.theme()
    
    # Acceder a servicios (DI)
    userService: UserService = context.service(UserService)
    
    # Acceder al widget padre
    parent: Widget? = context.findAncestorWidgetOfType<Container>()
    
    # Acceder al tamaño disponible
    size: Size = context.size()
    
    # Navegar
    context.navigate("/home")
    
    # Mostrar dialog
    context.showDialog(MyDialog())
    
    # Mostrar snackbar
    context.showSnackbar("Success!")
    
    return Container()
  }
}
```

**Provee acceso a:**
- ✅ Theme (colores, fonts, spacing)
- ✅ Services (DI container)
- ✅ Navigation (router)
- ✅ Layout constraints (size)
- ✅ Árbol de widgets (parent, ancestors)
- ✅ Dialogs y notificaciones

### 6. Element (Estado Mutable Interno)

**Arquitectura de 3 capas:**

```
Widget Tree (Inmutable)
    ↓
Element Tree (Mutable) ← Estado persiste aquí
    ↓
RenderObject Tree (Rendering)
```

**Element gestiona:**
- ✅ Referencia al widget actual
- ✅ Lifecycle (mounted, dirty)
- ✅ Parent/child relationships
- ✅ Scheduling de rebuilds
- ✅ Reconciliación (reusar vs reemplazar)

**Ejemplo interno:**
```vela
element: Element = Element(widget, parent, context)
element.mount()        # Montar en árbol
element.rebuild()      # Rebuild cuando state cambia
element.unmount()      # Desmontar del árbol
```

### 7. Scheduler (Batching de Rebuilds)

```vela
singleton WidgetScheduler {
  # Agrupar rebuilds en frames para eficiencia
  fn scheduleRebuild(element: Element) -> void {
    # Agregar a cola
    # Ejecutar en próximo frame (requestAnimationFrame)
  }
}
```

**Optimización:**
- ✅ Agrupa múltiples rebuilds en un solo frame
- ✅ Evita rebuilds innecesarios
- ✅ Usa requestAnimationFrame para 60fps

## ✅ Criterios de Aceptación

- [x] Widget base class implementada con lifecycle completo
- [x] StatelessWidget implementado (props inmutables únicamente)
- [x] StatefulWidget implementado (con state reactivo)
- [x] Key system (ValueKey, ObjectKey, GlobalKey)
- [x] BuildContext con acceso a theme, services, navigation
- [x] Element con ciclo de vida completo (mount, rebuild, unmount)
- [x] Scheduler para batching de rebuilds
- [x] 25+ tests unitarios pasando
- [x] Documentación completa con ejemplos
- [x] Integración con sistema reactivo (Signals)

## 📊 Métricas

### Código Implementado
- **Archivos creados:** 2
  - `src/ui/widget.vela` (650 líneas)
  - `tests/unit/ui/test_widget.vela` (450 líneas)
- **Clases:** 8
  - Widget (abstracta)
  - StatelessWidget (abstracta)
  - StatefulWidget (abstracta)
  - ValueKey, ObjectKey, GlobalKey
  - Element
  - WidgetScheduler (singleton)
- **Interfaces:** 1 (BuildContext)
- **Structs:** 7 (Size, Theme, TextTheme, TextStyle, Color, etc.)

### Tests
- **Total tests:** 25
- **Cobertura:**
  - Key system: 4 tests
  - StatelessWidget: 3 tests
  - StatefulWidget: 2 tests
  - Lifecycle hooks: 5 tests
  - Element: 4 tests
  - BuildContext: 3 tests
  - Composición: 2 tests
  - Keys en listas: 2 tests

### Performance
- **Widget creation:** O(1) - Muy ligero
- **canUpdate check:** O(1) - Comparación simple
- **Rebuild scheduling:** O(1) amortizado - Batch processing

## 🔗 Referencias

- **Archivo:** `src/ui/widget.vela`
- **Tests:** `tests/unit/ui/test_widget.vela`
- **ADR:** `docs/architecture/ADR-020-widget-architecture.md`
- **Depende de:** TASK-053 (Diseño de arquitectura)
- **Jira:** VELA-575

## 📈 Próximos Pasos

1. ✅ TASK-053: Diseño completo
2. ✅ TASK-054: Widget base class implementado
3. 🔄 TASK-055: Implementar widgets de layout (Container, Row, Column, Stack)
4. 🔄 TASK-056: Implementar widgets de input (Button, TextField, Checkbox)
5. 🔄 TASK-057: Implementar widgets de display (Text, Image, Icon)

## 💡 Ejemplos de Uso

### Ejemplo 1: StatelessWidget Simple

```vela
widget Greeting extends StatelessWidget {
  name: String
  
  fn build(context: BuildContext) -> Widget {
    return Text("Hello, ${this.name}!")
  }
}

# Uso
widget = Greeting { name: "Alice" }
```

### Ejemplo 2: StatefulWidget con Counter

```vela
widget Counter extends StatefulWidget {
  state count: Number = 0
  
  fn build(context: BuildContext) -> Widget {
    return Column(
      children: [
        Text("Count: ${this.count}"),
        Button(
          text: "Increment",
          onClick: () => this.count += 1
        )
      ]
    )
  }
}
```

### Ejemplo 3: Widget con Lifecycle

```vela
widget DataFetcher extends StatefulWidget {
  userId: String
  state data: User? = None
  state loading: Bool = false
  
  mount() -> void {
    this.loading = true
    fetchUser(this.userId).then(user => {
      this.data = Some(user)
      this.loading = false
    })
  }
  
  fn build(context: BuildContext) -> Widget {
    if this.loading {
      return LoadingSpinner()
    }
    
    match this.data {
      Some(user) => UserProfile { user: user }
      None => Text("No data")
    }
  }
  
  destroy() -> void {
    # Cancel pending requests
    cancelFetch(this.userId)
  }
}
```

### Ejemplo 4: Widget con Keys (Lista Dinámica)

```vela
widget TodoList extends StatefulWidget {
  state items: List<Todo> = []
  
  fn build(context: BuildContext) -> Widget {
    return Column(
      children: this.items.map(item => TodoItem {
        key: ValueKey(item.id),  # Key única por item
        todo: item,
        onDelete: () => this.deleteItem(item.id)
      })
    )
  }
  
  fn deleteItem(id: String) -> void {
    this.items = this.items.filter(item => item.id != id)
  }
}
```

### Ejemplo 5: Widget con BuildContext

```vela
widget ThemedWidget extends StatelessWidget {
  fn build(context: BuildContext) -> Widget {
    theme: Theme = context.theme()
    userService: UserService = context.service(UserService)
    
    return Container(
      color: theme.backgroundColor,
      child: Column(
        children: [
          Text(
            "Welcome, ${userService.currentUser().name}",
            style: theme.textTheme.headline1
          ),
          Button(
            text: "Navigate",
            onClick: () => context.navigate("/home")
          )
        ]
      )
    )
  }
}
```

---

**Completado:** 2025-12-05  
**Tiempo:** 24 horas  
**Estado:** ✅ Done  
**Bloqueadores:** Ninguno  
**Siguiente:** TASK-055 (Layout Widgets)
