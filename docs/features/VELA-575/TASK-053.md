# TASK-053: Diseñar Arquitectura de Widgets

## 📋 Información General
- **Historia:** VELA-575 (Sprint 20 - UI Framework)
- **Estado:** ✅ Completada
- **Fecha:** 2025-12-05
- **Prioridad:** P0
- **Estimación:** 32 horas

## 🎯 Objetivo

Diseñar el sistema de composición de widgets para el UI framework de Vela, estableciendo las bases arquitectónicas para un framework declarativo y reactivo inspirado en Flutter, React y SwiftUI.

## 🔨 Decisiones Arquitectónicas

### 1. Arquitectura de 3 Capas (Flutter-style)

**Decisión**: Separar declaración, estado y rendering en 3 capas distintas.

```
Widget Tree (Inmutable) → Element Tree (Mutable) → RenderObject Tree (Rendering)
```

**Capa 1: Widget Tree**
- **Propósito**: Declaración inmutable de la UI
- **Características**: Ligera, sin estado, recreada en cada rebuild
- **Ejemplo**: `Container(child: Text("Hello"))`

**Capa 2: Element Tree**
- **Propósito**: Estado mutable, gestión de ciclo de vida
- **Características**: Persiste entre rebuilds, mantiene estado
- **Gestión**: Interna del framework (no expuesta al desarrollador)

**Capa 3: RenderObject Tree**
- **Propósito**: Layout, painting, hit testing
- **Características**: Optimizado para rendering
- **Gestión**: Interna del framework

### 2. Sistema de Tipos de Widgets

```vela
# Widget base (abstracto)
abstract class Widget {
  key: Key?
  
  abstract fn build(context: BuildContext) -> Widget
}

# StatelessWidget: Sin estado mutable
abstract class StatelessWidget extends Widget {
  # Props inmutables únicamente
  # build() llamado cuando props cambian
}

# StatefulWidget: Con estado mutable
abstract class StatefulWidget extends Widget {
  # Puede tener state variables
  # build() llamado cuando state cambia
}
```

**Reglas de Uso:**
- Usa `StatelessWidget` si NO necesitas `state` (99% de widgets)
- Usa `StatefulWidget` si necesitas `state` mutable
- `state` variables triggean rebuild automático al mutar

### 3. Ciclo de Vida de Widgets

```vela
widget Counter extends StatefulWidget {
  state count: Number = 0
  
  # FASE 1: INICIALIZACIÓN
  constructor() {
    # Llamado UNA VEZ al crear
  }
  
  # FASE 2: MONTAJE
  mount() -> void {
    # Llamado al insertar en el árbol
    # Aquí: fetch data, timers, suscripciones
  }
  
  # FASE 3: ACTUALIZACIÓN (Loop)
  beforeUpdate() -> void {
    # Antes de rebuild
  }
  
  fn build() -> Widget {
    # Cada state change
    return Text("${this.count}")
  }
  
  afterUpdate() -> void {
    # Después de rebuild
  }
  
  # FASE 4: DESMONTAJE
  destroy() -> void {
    # Al remover del árbol
    # Aquí: cleanup, cancelar suscripciones
  }
}
```

**Orden de Ejecución:**
```
constructor() → mount() → [beforeUpdate() → build() → afterUpdate()]* → destroy()
```

### 4. Sistema de Keys

**Propósito**: Identificar widgets de forma única para reconciliación eficiente.

```vela
# Sin key: Framework puede confundir elementos
Column(
  children: [TextField(), TextField(), TextField()]
)

# Con key: Framework trackea correctamente
Column(
  children: items.map(item => TextField(
    key: ValueKey(item.id)  # Key única
  ))
)
```

**Tipos de Keys:**
```vela
# ValueKey: Key basada en valor primitivo
ValueKey(42)
ValueKey("user-123")

# ObjectKey: Key basada en identidad de objeto
ObjectKey(myObject)

# GlobalKey: Key global (acceso desde cualquier lugar)
GlobalKey()
```

**Cuándo usar Keys:**
- ✅ Listas dinámicas (añadir/remover/reordenar)
- ✅ Animaciones de entrada/salida
- ✅ Preservar estado al reordenar
- ❌ Listas estáticas (innecesario)

### 5. BuildContext

**Propósito**: Proveer acceso al árbol de widgets y servicios.

```vela
widget MyWidget extends StatelessWidget {
  fn build(context: BuildContext) -> Widget {
    # Acceder al theme
    theme: Theme = context.theme()
    
    # Acceder a servicios (DI)
    service: UserService = context.service(UserService)
    
    # Acceder al widget padre
    parent: Widget? = context.findAncestorWidgetOfType<Container>()
    
    # Acceder al tamaño disponible
    size: Size = context.size()
    
    # Navegar
    context.navigate("/home")
    
    return Container()
  }
}
```

**BuildContext provee:**
- Theme (colors, fonts, spacing)
- Services (DI container)
- Navigation (router)
- Constraints (layout)
- Árbol de widgets (parent, ancestors)

### 6. Sistema de Eventos

```vela
widget LoginForm extends StatefulWidget {
  state email: String = ""
  state password: String = ""
  
  fn handleSubmit() -> void {
    if this.email.isEmpty() {
      showError("Email required")
      return
    }
    
    login(this.email, this.password)
  }
  
  fn build() -> Widget {
    return Column(
      children: [
        TextField(
          value: this.email,
          placeholder: "Email",
          onChange: (value: String) => {
            this.email = value  # Muta state → rebuild
          },
          onFocus: () => print("Email focused"),
          onBlur: () => print("Email blurred")
        ),
        
        TextField(
          value: this.password,
          type: "password",
          placeholder: "Password",
          onChange: (value: String) => {
            this.password = value
          }
        ),
        
        Button(
          text: "Login",
          onClick: this.handleSubmit,
          disabled: this.email.isEmpty() || this.password.isEmpty()
        )
      ]
    )
  }
}
```

**Eventos Soportados:**

| Categoría | Eventos |
|-----------|---------|
| **Mouse** | onClick, onDoubleClick, onHover, onMouseEnter, onMouseLeave, onMouseDown, onMouseUp |
| **Keyboard** | onKeyDown, onKeyUp, onKeyPress |
| **Focus** | onFocus, onBlur |
| **Input** | onChange, onInput, onSubmit |
| **Touch** | onTap, onLongPress, onSwipe, onPinch |
| **Drag** | onDrag, onDragStart, onDragEnd, onDrop |
| **Scroll** | onScroll, onScrollEnd |

### 7. Composición de Widgets

**Principio Fundamental**: Widgets se componen de otros widgets.

```vela
# Widget atómico (no se descompone más)
widget Icon extends StatelessWidget {
  name: String
  size: Number = 24
  color: Color = Colors.Black
  
  fn build() -> Widget {
    # Renderiza SVG directamente
    return RawSVG(path: this.getIconPath())
  }
}

# Widget compuesto (compuesto de otros widgets)
widget IconButton extends StatelessWidget {
  icon: String
  onClick: () -> void
  size: Number = 48
  
  fn build() -> Widget {
    return Container(
      width: this.size,
      height: this.size,
      decoration: BoxDecoration(
        shape: BoxShape.Circle,
        color: Colors.Grey200
      ),
      child: Icon(
        name: this.icon,
        size: this.size * 0.5,
        color: Colors.Black
      ),
      onClick: this.onClick
    )
  }
}

# Widget de alto nivel (compuesto de múltiples widgets)
widget UserProfile extends StatefulWidget {
  userId: String
  state user: User? = None
  state loading: Bool = false
  
  mount() -> void {
    this.loading = true
    fetchUser(this.userId).then(user => {
      this.user = Some(user)
      this.loading = false
    })
  }
  
  fn build() -> Widget {
    if this.loading {
      return LoadingSpinner()
    }
    
    match this.user {
      Some(user) => {
        return Column(
          children: [
            # Avatar
            Container(
              width: 100,
              height: 100,
              decoration: BoxDecoration(
                shape: BoxShape.Circle,
                image: DecorationImage(url: user.avatarUrl)
              )
            ),
            
            # Name
            Text(
              user.name,
              style: TextStyle(fontSize: 24, weight: FontWeight.Bold)
            ),
            
            # Email
            Text(
              user.email,
              style: TextStyle(fontSize: 16, color: Colors.Grey)
            ),
            
            # Actions
            Row(
              children: [
                IconButton(
                  icon: "edit",
                  onClick: () => editUser(user.id)
                ),
                IconButton(
                  icon: "delete",
                  onClick: () => deleteUser(user.id)
                )
              ]
            )
          ]
        )
      }
      None => Text("User not found")
    }
  }
}
```

**Reglas de Composición:**
1. ✅ Widget puede contener múltiples hijos
2. ✅ Hijos pueden ser cualquier widget
3. ✅ Anidación ilimitada
4. ⚠️ Evitar anidación excesiva (>10 niveles) por performance

### 8. Sistema de Estilos

```vela
# Estilos inline
Text(
  "Hello World",
  style: TextStyle(
    fontSize: 24,
    color: Color.fromRGB(255, 0, 0),
    fontWeight: FontWeight.Bold,
    fontFamily: "Roboto"
  )
)

# Theme global
widget App extends StatelessWidget {
  fn build() -> Widget {
    return Theme(
      data: ThemeData(
        primaryColor: Color.Blue,
        secondaryColor: Color.Green,
        errorColor: Color.Red,
        backgroundColor: Color.White,
        textTheme: TextTheme(
          headline1: TextStyle(fontSize: 32, weight: FontWeight.Bold),
          headline2: TextStyle(fontSize: 24, weight: FontWeight.Bold),
          body1: TextStyle(fontSize: 16),
          body2: TextStyle(fontSize: 14),
          caption: TextStyle(fontSize: 12, color: Colors.Grey)
        ),
        buttonTheme: ButtonThemeData(
          height: 48,
          borderRadius: 8,
          backgroundColor: Color.Blue,
          textColor: Color.White
        )
      ),
      child: MyApp()
    )
  }
}

# Usar theme
widget StyledWidget extends StatelessWidget {
  fn build(context: BuildContext) -> Widget {
    theme: Theme = context.theme()
    
    return Container(
      color: theme.backgroundColor,
      child: Column(
        children: [
          Text(
            "Title",
            style: theme.textTheme.headline1
          ),
          Text(
            "Body",
            style: theme.textTheme.body1
          ),
          Button(
            text: "Action",
            style: theme.buttonTheme
          )
        ]
      )
    )
  }
}
```

## ✅ Criterios de Aceptación

- [x] ADR-020 creado con decisiones arquitectónicas
- [x] Definición de Widget, StatelessWidget, StatefulWidget
- [x] Ciclo de vida completo diseñado
- [x] Sistema de Keys definido
- [x] BuildContext especificado
- [x] Sistema de eventos documentado
- [x] Composición de widgets explicada
- [x] Sistema de estilos diseñado
- [x] Ejemplos de código completos
- [x] Referencias a Flutter/React/SwiftUI

## 📊 Métricas

- **Decisiones Arquitectónicas:** 8
- **Ejemplos de Código:** 15+
- **Tipos de Widgets:** 2 (Stateless, Stateful)
- **Fases de Ciclo de Vida:** 4 (init, mount, update, destroy)
- **Tipos de Keys:** 3 (Value, Object, Global)
- **Eventos Soportados:** 25+

## 🔗 Referencias

- **ADR:** `docs/architecture/ADR-020-widget-architecture.md`
- **Jira:** VELA-575 (Sprint 20)
- **Flutter Widgets:** https://flutter.dev/docs/development/ui/widgets
- **React Hooks:** https://react.dev/reference/react
- **SwiftUI Views:** https://developer.apple.com/documentation/swiftui

## 📁 Archivos Generados

```
docs/architecture/ADR-020-widget-architecture.md     (11 KB)
docs/features/VELA-575/TASK-053.md                   (este archivo)
```

## 🚀 Próximos Pasos

1. ✅ TASK-053: Diseño completo
2. 🔄 TASK-054: Implementar Widget base class
3. 🔄 TASK-055: Implementar widgets de layout
4. 🔄 TASK-056: Implementar widgets de input
5. 🔄 TASK-057: Implementar widgets de display

---

**Completado:** 2025-12-05  
**Tiempo:** 32 horas (estimado)  
**Estado:** ✅ Done
