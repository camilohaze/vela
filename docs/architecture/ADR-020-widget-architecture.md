# ADR-020: Arquitectura de Widgets para UI Framework

## Estado
✅ Aceptado

## Fecha
2025-12-05

## Contexto

Vela necesita un sistema de UI declarativo y reactivo para construir interfaces de usuario modernas. El sistema debe:

1. **Ser declarativo**: Describir CÓMO se ve la UI, no cómo construirla
2. **Ser reactivo**: Actualizarse automáticamente cuando el estado cambia
3. **Ser componible**: Widgets dentro de widgets
4. **Ser eficiente**: Minimizar actualizaciones del DOM real
5. **Ser tipado**: Type-safe en tiempo de compilación
6. **Soportar múltiples plataformas**: Web, Mobile, Desktop

### Referencias de Arquitectura

**Flutter (Dart):**
- Widget tree con StatefulWidget y StatelessWidget
- Elemento tree separado del widget tree
- RenderObject tree para layout y painting
- InheritedWidget para propagación de estado

**React (JavaScript/TypeScript):**
- Componentes funcionales y de clase
- Hooks para estado y efectos
- Virtual DOM con reconciliación
- Context API para estado global

**SwiftUI (Swift):**
- Structs ligeros para vistas
- Property wrappers (@State, @Binding, @ObservedObject)
- View protocol con body computed property
- Environment para datos compartidos

**Angular (TypeScript):**
- Componentes con decoradores @Component
- Change detection con zones
- Templates declarativos
- Dependency injection integrado

## Decisión

### 1. **Arquitectura de 3 Capas (Inspirado en Flutter)**

```vela
# CAPA 1: Widget Tree (Declaración inmutable)
widget Counter {
  state count: Number = 0
  
  fn build() -> Widget {
    return Container(
      child: Column(
        children: [
          Text("Count: ${this.count}"),
          Button(
            text: "Increment",
            onClick: () => this.count += 1
          )
        ]
      )
    )
  }
}

# CAPA 2: Element Tree (Estado mutable, maneja ciclo de vida)
# Gestionado internamente por el framework

# CAPA 3: RenderObject Tree (Layout y painting)
# Gestionado internamente por el framework
```

**Justificación**: Flutter demostró que separar declaración (widgets) de estado (elements) de rendering (render objects) es más eficiente y escalable.

---

### 2. **Sistema de Widgets: StatefulWidget vs StatelessWidget**

```vela
# Widget sin estado (puro, sin state)
widget Label extends StatelessWidget {
  text: String
  fontSize: Number = 14
  
  fn build() -> Widget {
    return Text(
      this.text,
      style: TextStyle(fontSize: this.fontSize)
    )
  }
}

# Widget con estado (puede mutar)
widget Counter extends StatefulWidget {
  state count: Number = 0
  
  fn build() -> Widget {
    return Column(
      children: [
        Text("Count: ${this.count}"),
        Button(
          text: "+1",
          onClick: () => this.count += 1  # Muta state, triggerea rebuild
        )
      ]
    )
  }
}
```

**Regla de Oro**: 
- `StatelessWidget` → Sin `state`, solo props inmutables
- `StatefulWidget` → Con `state`, puede mutar y trigger rebuilds

---

### 3. **Ciclo de Vida de Widgets (Inspirado en React + Flutter)**

```vela
widget MyWidget extends StatefulWidget {
  state data: String = ""
  
  # 1. INICIALIZACIÓN
  constructor(initialData: String) {
    # Constructor llamado UNA VEZ
  }
  
  # 2. MONTAJE
  mount() -> void {
    # Llamado después de insertar en el árbol
    # Aquí: fetch data, suscripciones, timers
    this.data = fetchData()
  }
  
  # 3. ACTUALIZACIÓN
  beforeUpdate() -> void {
    # Llamado ANTES de rebuild
    # Aquí: comparar props viejos vs nuevos
  }
  
  fn build() -> Widget {
    # Llamado cada vez que state cambia
    return Text(this.data)
  }
  
  afterUpdate() -> void {
    # Llamado DESPUÉS de rebuild
    # Aquí: side effects post-render
  }
  
  # 4. DESMONTAJE
  destroy() -> void {
    # Llamado al remover del árbol
    # Aquí: cleanup, cancelar suscripciones
  }
}
```

**Orden de ejecución:**
1. `constructor()` → UNA VEZ al crear instancia
2. `mount()` → UNA VEZ al insertar en árbol
3. Loop: `beforeUpdate()` → `build()` → `afterUpdate()` (cada state change)
4. `destroy()` → UNA VEZ al remover del árbol

---

### 4. **Sistema de Keys (Identificación de Widgets)**

```vela
# Sin key: React puede confundir elementos al reordenar
Column(
  children: [
    TextField(value: "Item 1"),
    TextField(value: "Item 2"),
    TextField(value: "Item 3")
  ]
)

# Con key: React puede trackear elementos correctamente
Column(
  children: items.map(item => TextField(
    key: item.id,  # Key única por elemento
    value: item.text
  ))
)
```

**Tipos de Keys:**
- `ValueKey(value)` - Key basada en valor primitivo
- `ObjectKey(object)` - Key basada en identidad de objeto
- `GlobalKey()` - Key única globalmente (acceso desde cualquier lugar)

---

### 5. **BuildContext (Acceso al Árbol de Widgets)**

```vela
widget MyWidget extends StatelessWidget {
  fn build(context: BuildContext) -> Widget {
    # Acceder al theme
    theme: Theme = context.theme()
    
    # Acceder a servicios inyectados
    userService: UserService = context.service(UserService)
    
    # Acceder al widget padre
    parent: Widget? = context.parent()
    
    # Acceder al tamaño disponible
    size: Size = context.size()
    
    return Container(
      color: theme.primaryColor,
      child: Text("Hello")
    )
  }
}
```

**BuildContext provee:**
- Acceso al theme (colores, fonts, etc.)
- Acceso a servicios (DI)
- Acceso al árbol de widgets
- Acceso a constraints de layout
- Navegación (context.navigate())

---

### 6. **Sistema de Eventos (Inspirado en React)**

```vela
widget LoginForm extends StatefulWidget {
  state email: String = ""
  state password: String = ""
  
  fn build() -> Widget {
    return Column(
      children: [
        TextField(
          value: this.email,
          onChange: (newValue: String) => {
            this.email = newValue  # State mutation
          },
          onFocus: () => print("Email focused"),
          onBlur: () => print("Email blurred")
        ),
        
        TextField(
          value: this.password,
          type: "password",
          onChange: (newValue: String) => {
            this.password = newValue
          }
        ),
        
        Button(
          text: "Login",
          onClick: () => {
            login(this.email, this.password)
          },
          onHover: () => print("Hovering button"),
          disabled: this.email.isEmpty() || this.password.isEmpty()
        )
      ]
    )
  }
}
```

**Eventos soportados:**
- `onClick`, `onDoubleClick`, `onLongPress`
- `onChange`, `onInput`, `onSubmit`
- `onFocus`, `onBlur`
- `onHover`, `onMouseEnter`, `onMouseLeave`
- `onKeyDown`, `onKeyUp`, `onKeyPress`
- `onDrag`, `onDrop`

---

### 7. **Composición de Widgets (Patrón Fundamental)**

```vela
# Widget compuesto de otros widgets
widget UserCard extends StatelessWidget {
  user: User
  onTap: () -> void
  
  fn build() -> Widget {
    return Container(
      padding: EdgeInsets.all(16),
      child: Row(
        children: [
          # Avatar
          Container(
            width: 48,
            height: 48,
            decoration: BoxDecoration(
              shape: BoxShape.Circle,
              image: DecorationImage(url: this.user.avatarUrl)
            )
          ),
          
          # Info
          Column(
            crossAxisAlignment: CrossAxisAlignment.Start,
            children: [
              Text(
                this.user.name,
                style: TextStyle(fontSize: 16, weight: FontWeight.Bold)
              ),
              Text(
                this.user.email,
                style: TextStyle(fontSize: 14, color: Colors.Grey)
              )
            ]
          ),
          
          # Action button
          Button(
            text: "Follow",
            onClick: this.onTap
          )
        ]
      )
    )
  }
}

# Uso del widget compuesto
widget UserList extends StatefulWidget {
  state users: List<User> = []
  
  fn build() -> Widget {
    return Column(
      children: this.users.map(user => UserCard(
        key: ValueKey(user.id),
        user: user,
        onTap: () => followUser(user.id)
      ))
    )
  }
}
```

---

### 8. **Sistema de Estilos (CSS-like pero tipado)**

```vela
# Estilos inline
widget StyledText extends StatelessWidget {
  fn build() -> Widget {
    return Text(
      "Hello World",
      style: TextStyle(
        fontSize: 24,
        color: Color.fromRGB(255, 0, 0),
        fontWeight: FontWeight.Bold,
        fontFamily: "Roboto",
        letterSpacing: 1.2,
        lineHeight: 1.5
      )
    )
  }
}

# Estilos compartidos (Theme)
widget ThemedApp extends StatelessWidget {
  fn build() -> Widget {
    return Theme(
      data: ThemeData(
        primaryColor: Color.Blue,
        secondaryColor: Color.Green,
        textTheme: TextTheme(
          headline1: TextStyle(fontSize: 32, weight: FontWeight.Bold),
          headline2: TextStyle(fontSize: 24, weight: FontWeight.Bold),
          body1: TextStyle(fontSize: 16),
          body2: TextStyle(fontSize: 14)
        )
      ),
      child: MyApp()
    )
  }
}

# Usar theme
widget MyWidget extends StatelessWidget {
  fn build(context: BuildContext) -> Widget {
    theme: Theme = context.theme()
    
    return Container(
      color: theme.primaryColor,
      child: Text(
        "Hello",
        style: theme.textTheme.headline1
      )
    )
  }
}
```

---

## Consecuencias

### Positivas

1. **✅ Declarativo y Componible**:
   - UI se declara como árbol de widgets
   - Fácil de leer y mantener
   - Reutilización de componentes

2. **✅ Reactivo y Eficiente**:
   - State changes triggean rebuilds automáticos
   - Virtual DOM minimiza actualizaciones reales
   - Diffing algorithm optimizado

3. **✅ Type-Safe**:
   - Tipos estrictos en tiempo de compilación
   - No hay "prop drilling" sin tipos
   - Autocomplete en IDEs

4. **✅ Multiplataforma**:
   - Web: Compila a HTML/CSS/JS
   - Mobile: Compila a widgets nativos (iOS/Android)
   - Desktop: Compila a widgets nativos (Windows/macOS/Linux)

5. **✅ Familiar**:
   - Desarrolladores de Flutter: Reconocen StatefulWidget/StatelessWidget
   - Desarrolladores de React: Reconocen hooks y ciclo de vida
   - Desarrolladores de Angular: Reconocen decoradores y DI

### Negativas

1. **❌ Curva de Aprendizaje**:
   - Conceptos de Virtual DOM y reconciliación
   - Diferencia entre StatefulWidget y StatelessWidget
   - Keys y optimizaciones

2. **❌ Overhead Inicial**:
   - Virtual DOM consume memoria
   - Diffing algorithm tiene costo computacional
   - Element tree adicional

3. **❌ Complejidad de Implementación**:
   - Reconciliación es compleja
   - Layout engine es complejo
   - Multi-platform rendering es complejo

---

## Alternativas Consideradas

### 1. **Imperative UI (jQuery-style)**

```javascript
// ❌ RECHAZADO: Demasiado verboso y propenso a errores
const button = document.createElement('button');
button.textContent = 'Click me';
button.addEventListener('click', handleClick);
document.body.appendChild(button);
```

**Rechazado porque:**
- No es declarativo
- Estado no sincronizado con UI
- Difícil de mantener

### 2. **Template-based (Angular/Vue)**

```html
<!-- ❌ RECHAZADO: Separa lógica de presentación -->
<div *ngFor="let item of items">
  <span>{{ item.name }}</span>
</div>
```

**Rechazado porque:**
- Templates en strings/HTML separados
- No hay type-safety en templates
- Sintaxis especial difícil de parsear

### 3. **JSX/TSX (React-style)**

```tsx
// ⚠️ CONSIDERADO pero no adoptado
const Counter = () => {
  const [count, setCount] = useState(0);
  return (
    <div>
      <span>{count}</span>
      <button onClick={() => setCount(count + 1)}>+</button>
    </div>
  );
};
```

**No adoptado porque:**
- Vela no usa XML/HTML syntax
- Preferimos sintaxis nativa del lenguaje
- Pero: Los conceptos de React SÍ se adoptaron (hooks, components)

---

## Referencias

- **Flutter Widget Catalog**: https://flutter.dev/docs/development/ui/widgets
- **React Documentation**: https://react.dev/
- **SwiftUI Tutorials**: https://developer.apple.com/tutorials/swiftui
- **Angular Components**: https://angular.io/guide/component-overview
- **Virtual DOM Explained**: https://github.com/Matt-Esch/virtual-dom
- **React Reconciliation**: https://react.dev/learn/preserving-and-resetting-state

---

## Implementación

### Sprint 20 (UI Framework Basics):
- ✅ TASK-053: Diseñar arquitectura de widgets (este ADR)
- 🔄 TASK-054: Implementar Widget base class
- 🔄 TASK-055: Implementar widgets de layout (Container, Row, Column, Stack)
- 🔄 TASK-056: Implementar widgets de input (Button, TextField, Checkbox)
- 🔄 TASK-057: Implementar widgets de display (Text, Image, Icon)

### Sprint 21 (Reactive UI):
- TASK-058: Integrar signals con widgets
- TASK-059: Implementar Virtual DOM
- TASK-060: Implementar diffing algorithm
- TASK-061: Implementar patching system
- TASK-062: Tests de reconciliación reactiva

---

**Autor:** GitHub Copilot Agent  
**Fecha:** 2025-12-05  
**Relacionado con:** VELA-575 (Sprint 20 - UI Framework)
