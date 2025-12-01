# 6. Definición de UI Declarativa

## 6.1 Filosofía de Diseño

### 6.1.1 Principios

**Inspiración**: Flutter, SwiftUI, React, Vue

**Características clave**:
1. **Declarativo**: Describe QUÉ mostrar, no CÓMO
2. **Reactivo**: UI se actualiza automáticamente con cambios de estado
3. **Composable**: Widgets pequeños componen widgets complejos
4. **Type-safe**: Verificación en compile-time
5. **Cross-platform**: Un código, múltiples plataformas

---

### 6.1.2 Modelo Mental

```
State (signals) → Build function → Virtual DOM → Renderer → Screen
     ↑                                                          ↓
     └──────────────────── Events ←─────────────────────────────┘
```

**Ciclo de vida**:
```
1. State cambia (signal.value = x)
2. Build function se re-ejecuta (tracked por signal)
3. Genera nuevo Virtual DOM
4. Diff con Virtual DOM anterior
5. Apply patches al DOM real
6. Render en pantalla
```

---

## 6.2 Widget System (Flutter-style)

### 6.2.1 StatelessWidget

```vela
public abstract class StatelessWidget {
  // Lifecycle hooks
  protected fn onMount(): void {}
  protected fn onUnmount(): void {}
  
  // Build method (abstract)
  public abstract fn build(): StatelessWidget;
  
  // Context API
  protected fn useContext<T>(key: ContextKey<T>): Option<T> {
    return Context.get<T>(key);
  }
}
```

### 6.2.2 StatefulWidget

```vela
public abstract class StatefulWidget {
  // Lifecycle hooks
  protected fn onMount(): void {}
  protected fn onUpdate(): void {}
  protected fn onUnmount(): void {}
  
  // Build method (abstract)
  public abstract fn build(): StatefulWidget;
  
  // State management (interno - usa signals)
  protected fn useState<T>(initial: T): Signal<T> {
    return signal(initial);
  }
  
  protected fn useEffect(effect: () => void): void {
    effect(() => {
      effect();
    });
  }
  
  protected fn useMemo<T>(compute: () => T): Computed<T> {
    return computed(compute);
  }
  
  // Context API
  protected fn useContext<T>(key: ContextKey<T>): Option<T> {
    return Context.get<T>(key);
  }
  
  protected fn provideContext<T>(key: ContextKey<T>, value: T): void {
    Context.provide<T>(key, value);
  }
}
```

---

### 6.2.3 Functional Widgets (Preferred)

```vela
// StatefulWidget funcional (con estado local)
fn Counter(): StatefulWidget {
  count = signal(0);  // Inmutable por defecto
  
  return Container {
    padding: EdgeInsets.all(20),
    
    Column {
      spacing: 16,
      
      Text("Count: ${count.value}"),
      
      Button {
        label: "Increment",
        onClick: () => state { count.value = count.value + 1; }
      }
    }
  };
}

// Class-based StatefulWidget (cuando necesitas lifecycle complejo)
class TimerWidget extends StatefulWidget {
  private elapsed = signal(0);
  private intervalId: Option<Int> = Option.None;
  
  fn onMount(): void {
    this.intervalId = Option.Some(setInterval(() => {
      state { this.elapsed.value = this.elapsed.value + 1; }
    }, 1000));
  }
  
  fn onUnmount(): void {
    match (this.intervalId) {
      Option.Some(id) => clearInterval(id),
      Option.None => {}
    }
  }
  
  fn build(): StatefulWidget {
    return Text("Elapsed: ${this.elapsed.value}s");
  }
}
```

---

## 6.3 Layout Widgets

### 6.3.1 Container

```vela
fn Container(
  child: Option<StatelessWidget>,
  children: Option<List<StatelessWidget>>,
  
  // Size
  width: Option<Float>,
  height: Option<Float>,
  minWidth: Option<Float>,
  minHeight: Option<Float>,
  maxWidth: Option<Float>,
  maxHeight: Option<Float>,
  
  // Spacing
  padding: Option<EdgeInsets>,
  margin: Option<EdgeInsets>,
  
  // Styling
  color: Option<Color>,
  backgroundColor: Option<Color>,
  border: Option<Border>,
  borderRadius: Option<BorderRadius>,
  boxShadow: Option<BoxShadow>,
  
  // Layout
  alignment: Option<Alignment>,
  
  // Events
  onClick: Option<() => void>
): StatelessWidget;

// Example
Container {
  width: 200,
  height: 100,
  padding: EdgeInsets.all(16),
  backgroundColor: Colors.blue,
  borderRadius: BorderRadius.circular(8),
  
  Text("Hello, Vela!")
}
```

---

### 6.3.2 Row (Horizontal Layout)

```vela
fn Row(
  children: List<StatelessWidget>,
  spacing: Option<Float>,
  mainAxisAlignment: Option<MainAxisAlignment>,  // start, end, center, spaceBetween, spaceAround
  crossAxisAlignment: Option<CrossAxisAlignment>, // start, end, center, stretch
  wrap: Option<Bool>
): StatelessWidget;

// Example
Row {
  spacing: 8,
  mainAxisAlignment: MainAxisAlignment.spaceBetween,
  
  Text("Left"),
  Text("Middle"),
  Text("Right")
}
```

---

### 6.3.3 Column (Vertical Layout)

```vela
fn Column(
  children: List<StatelessWidget>,
  spacing: Option<Float>,
  mainAxisAlignment: Option<MainAxisAlignment>,
  crossAxisAlignment: Option<CrossAxisAlignment>
): StatelessWidget;

// Example
Column {
  spacing: 16,
  crossAxisAlignment: CrossAxisAlignment.stretch,
  
  Text("Title", style: TextStyle(fontSize: 24, fontWeight: FontWeight.bold)),
  Text("Subtitle"),
  Divider(),
  Text("Content goes here...")
}
```

---

### 6.3.4 Stack (Z-axis Stacking)

```vela
fn Stack(
  children: List<StatelessWidget>,
  alignment: Option<Alignment>
): StatelessWidget;

// Example
Stack {
  alignment: Alignment.center,
  
  Image { src: "background.jpg" },
  Container {
    color: Colors.black.withOpacity(0.5),
    Text("Overlay text", style: TextStyle(color: Colors.white))
  }
}
```

---

### 6.3.5 Flex (Flexible Layout)

```vela
fn Flex(
  direction: FlexDirection,  // row, column
  children: List<Widget>,
  spacing: Float?,
  wrap: Bool?
): Widget;

fn Flexible(
  child: Widget,
  flex: Int?,    // Flex factor
  fit: FlexFit?  // loose, tight
): Widget;

fn Expanded(
  child: Widget,
  flex: Int?
): Widget;

// Example
Row {
  Expanded {
    flex: 2,
    Container { color: Colors.red, height: 50 }
  },
  Expanded {
    flex: 1,
    Container { color: Colors.blue, height: 50 }
  }
}
```

---

### 6.3.6 Grid

```vela
fn Grid(
  children: List<Widget>,
  columns: Int,
  rows: Int?,
  columnGap: Float?,
  rowGap: Float?,
  aspectRatio: Float?
): Widget;

// Example
Grid {
  columns: 3,
  columnGap: 8,
  rowGap: 8,
  
  for (i in 0..9) {
    Container {
      color: Colors.blue,
      alignment: Alignment.center,
      Text("${i}")
    }
  }
}
```

---

## 6.4 Input Widgets

### 6.4.1 Button

```vela
fn Button(
  label: String?,
  child: Widget?,
  onClick: () => void,
  disabled: Bool?,
  variant: ButtonVariant?,  // primary, secondary, outlined, text
  size: ButtonSize?,        // small, medium, large
  icon: IconData?,
  loading: Bool?
): Widget;

// Example
Button {
  label: "Click me",
  variant: ButtonVariant.primary,
  icon: Icons.check,
  onClick: () => {
    print("Button clicked!");
  }
}
```

---

### 6.4.2 TextField

```vela
fn TextField(
  value: Signal<String>,
  placeholder: String?,
  label: String?,
  multiline: Bool?,
  maxLines: Int?,
  maxLength: Int?,
  obscureText: Bool?,  // For passwords
  keyboardType: KeyboardType?,
  autocorrect: Bool?,
  onChanged: ((String) => void)?,
  onSubmit: ((String) => void)?,
  validator: ((String) => String?)?,
  decoration: InputDecoration?
): Widget;

// Example
fn LoginForm(): Widget {
  let username = signal("");
  let password = signal("");
  
  return Column {
    spacing: 16,
    
    TextField {
      value: username,
      label: "Username",
      placeholder: "Enter your username",
      onChanged: (value) => print("Username: ${value}")
    },
    
    TextField {
      value: password,
      label: "Password",
      obscureText: true,
      onSubmit: (value) => handleLogin()
    },
    
    Button {
      label: "Login",
      onClick: handleLogin
    }
  };
}
```

---

### 6.4.3 Checkbox & Switch

```vela
fn Checkbox(
  value: Signal<Bool>,
  label: String?,
  onChange: ((Bool) => void)?
): Widget;

fn Switch(
  value: Signal<Bool>,
  onChange: ((Bool) => void)?
): Widget;

// Example
fn SettingsPanel(): Widget {
  let notifications = signal(true);
  let darkMode = signal(false);
  
  return Column {
    spacing: 12,
    
    Checkbox {
      value: notifications,
      label: "Enable notifications"
    },
    
    Row {
      spacing: 8,
      Text("Dark mode"),
      Switch { value: darkMode }
    }
  };
}
```

---

### 6.4.4 Radio & RadioGroup

```vela
fn Radio<T>(
  groupValue: Signal<T>,
  value: T,
  label: String?,
  onChange: ((T) => void)?
): Widget;

fn RadioGroup<T>(
  value: Signal<T>,
  children: List<Widget>,
  onChange: ((T) => void)?
): Widget;

// Example
fn LanguageSelector(): Widget {
  let language = signal("en");
  
  return RadioGroup {
    value: language,
    onChange: (value) => print("Selected: ${value}"),
    
    Radio { groupValue: language, value: "en", label: "English" },
    Radio { groupValue: language, value: "es", label: "Español" },
    Radio { groupValue: language, value: "fr", label: "Français" }
  };
}
```

---

### 6.4.5 Dropdown & Select

```vela
fn Dropdown<T>(
  value: Signal<T>,
  items: List<DropdownItem<T>>,
  label: String?,
  onChange: ((T) => void)?
): Widget;

class DropdownItem<T> {
  public value: T;
  public label: String;
  public icon: IconData?;
}

// Example
fn CountrySelector(): Widget {
  let country = signal("us");
  
  return Dropdown {
    value: country,
    label: "Select country",
    items: [
      DropdownItem(value: "us", label: "United States"),
      DropdownItem(value: "uk", label: "United Kingdom"),
      DropdownItem(value: "ca", label: "Canada")
    ],
    onChange: (value) => print("Selected: ${value}")
  };
}
```

---

### 6.4.6 Slider

```vela
fn Slider(
  value: Signal<Float>,
  min: Float,
  max: Float,
  step: Float?,
  divisions: Int?,
  label: String?,
  onChange: ((Float) => void)?
): Widget;

// Example
fn VolumeControl(): Widget {
  let volume = signal(50.0);
  
  return Column {
    spacing: 8,
    
    Text("Volume: ${volume.value.toInt()}%"),
    
    Slider {
      value: volume,
      min: 0,
      max: 100,
      divisions: 10,
      onChange: (value) => setVolume(value)
    }
  };
}
```

---

## 6.5 Display Widgets

### 6.5.1 Text

```vela
fn Text(
  text: String,
  style: TextStyle?,
  textAlign: TextAlign?,
  maxLines: Int?,
  overflow: TextOverflow?
): Widget;

class TextStyle {
  public fontSize: Float?;
  public fontWeight: FontWeight?;
  public fontStyle: FontStyle?;
  public color: Color?;
  public backgroundColor: Color?;
  public decoration: TextDecoration?;
  public decorationColor: Color?;
  public letterSpacing: Float?;
  public wordSpacing: Float?;
  public lineHeight: Float?;
  public shadows: List<Shadow>?;
}

// Example
Text(
  "Hello, Vela!",
  style: TextStyle(
    fontSize: 24,
    fontWeight: FontWeight.bold,
    color: Colors.blue,
    decoration: TextDecoration.underline
  )
)
```

---

### 6.5.2 Image

```vela
fn Image(
  src: String,          // URL or local path
  width: Float?,
  height: Float?,
  fit: ImageFit?,       // cover, contain, fill, fitWidth, fitHeight
  alignment: Alignment?,
  placeholder: Widget?,
  errorWidget: Widget?,
  loading: Widget?
): Widget;

// Example
Image {
  src: "https://example.com/image.jpg",
  width: 200,
  height: 200,
  fit: ImageFit.cover,
  placeholder: Spinner(),
  errorWidget: Icon(Icons.error)
}
```

---

### 6.5.3 Icon

```vela
fn Icon(
  icon: IconData,
  size: Float?,
  color: Color?
): Widget;

class Icons {
  public static home: IconData;
  public static search: IconData;
  public static settings: IconData;
  public static user: IconData;
  public static check: IconData;
  public static close: IconData;
  // ... hundreds more
}

// Example
Icon(Icons.home, size: 24, color: Colors.blue)
```

---

### 6.5.4 Progress Indicators

```vela
fn CircularProgressIndicator(
  size: Float?,
  strokeWidth: Float?,
  color: Color?,
  value: Float?  // null = indeterminate, 0.0-1.0 = determinate
): Widget;

fn LinearProgressIndicator(
  value: Float?,
  minHeight: Float?,
  color: Color?,
  backgroundColor: Color?
): Widget;

// Example
fn LoadingScreen(): Widget {
  return Container {
    alignment: Alignment.center,
    
    Column {
      spacing: 16,
      
      CircularProgressIndicator(),
      Text("Loading...")
    }
  };
}
```

---

## 6.6 Scrolling & Lists

### 6.6.1 ScrollView

```vela
fn ScrollView(
  child: Widget,
  direction: ScrollDirection?,  // vertical, horizontal
  scrollbar: Bool?
): Widget;

// Example
ScrollView {
  Column {
    for (i in 0..100) {
      Text("Item ${i}")
    }
  }
}
```

---

### 6.6.2 ListView (Virtualized)

```vela
fn ListView<T>(
  items: List<T>,
  builder: (T, Int) => Widget,
  separator: Widget?,
  scrollDirection: ScrollDirection?,
  padding: EdgeInsets?
): Widget;

// Example
fn UserList(users: List<User>): Widget {
  return ListView(
    items: users,
    builder: (user, index) => {
      return Container {
        padding: EdgeInsets.all(16),
        
        Row {
          spacing: 12,
          
          Image { src: user.avatar, width: 48, height: 48 },
          
          Column {
            crossAxisAlignment: CrossAxisAlignment.start,
            
            Text(user.name, style: TextStyle(fontWeight: FontWeight.bold)),
            Text(user.email, style: TextStyle(color: Colors.gray))
          }
        }
      };
    },
    separator: Divider()
  );
}
```

---

### 6.6.3 GridView

```vela
fn GridView<T>(
  items: List<T>,
  builder: (T, Int) => Widget,
  columns: Int,
  aspectRatio: Float?,
  spacing: Float?
): Widget;

// Example
fn PhotoGallery(photos: List<Photo>): Widget {
  return GridView(
    items: photos,
    columns: 3,
    aspectRatio: 1.0,
    spacing: 8,
    builder: (photo, index) => {
      return Image { src: photo.url, fit: ImageFit.cover };
    }
  );
}
```

---

## 6.7 Navigation & Routing

### 6.7.1 Router

```vela
fn Router(
  routes: Dict<String, () => Widget>,
  initialRoute: String?,
  notFoundRoute: (() => Widget)?
): Widget;

// Example
fn App(): Widget {
  return Router(
    routes: {
      "/": () => HomePage(),
      "/about": () => AboutPage(),
      "/users/:id": () => UserDetailPage(),
      "/settings": () => SettingsPage()
    },
    initialRoute: "/",
    notFoundRoute: () => NotFoundPage()
  );
}
```

---

### 6.7.2 Navigation API

```vela
class Navigation {
  public static fn push(route: String, params: Dict<String, any>?): void;
  public static fn pop(): void;
  public static fn replace(route: String, params: Dict<String, any>?): void;
  public static fn popUntil(route: String): void;
  public static fn getCurrentRoute(): String;
  public static fn getParams(): Dict<String, any>;
}

// Example
Button {
  label: "Go to User",
  onClick: () => Navigation.push("/users/123")
}
```

---

## 6.8 Animations

### 6.8.1 Animated Widget

```vela
fn Animated(
  child: Widget,
  duration: Duration,
  curve: AnimationCurve?,
  
  // Animatable properties
  opacity: AnimatedValue<Float>?,
  scale: AnimatedValue<Float>?,
  rotation: AnimatedValue<Float>?,
  translate: AnimatedValue<(Float, Float)>?,
  
  onComplete: (() => void)?
): Widget;

// Example
fn FadeIn(child: Widget): Widget {
  let opacity = signal(0.0);
  
  useEffect(() => {
    state { opacity.value = 1.0; }
  });
  
  return Animated {
    child: child,
    opacity: opacity,
    duration: Duration.milliseconds(500),
    curve: AnimationCurve.easeIn
  };
}
```

---

### 6.8.2 Transition Widgets

```vela
fn FadeTransition(child: Widget, opacity: Signal<Float>): Widget;
fn ScaleTransition(child: Widget, scale: Signal<Float>): Widget;
fn SlideTransition(child: Widget, offset: Signal<(Float, Float)>): Widget;
fn RotateTransition(child: Widget, rotation: Signal<Float>): Widget;

// Example
fn AnimatedButton(): Widget {
  let scale = signal(1.0);
  
  return ScaleTransition {
    scale: scale,
    
    Button {
      label: "Hover me",
      onHover: () => state { scale.value = 1.1; },
      onHoverEnd: () => state { scale.value = 1.0; }
    }
  };
}
```

---

## 6.9 Gestures

```vela
fn GestureDetector(
  child: Widget,
  onClick: (() => void)?,
  onDoubleClick: (() => void)?,
  onLongPress: (() => void)?,
  onHover: (() => void)?,
  onHoverEnd: (() => void)?,
  onDragStart: ((DragDetails) => void)?,
  onDragUpdate: ((DragDetails) => void)?,
  onDragEnd: ((DragDetails) => void)?
): Widget;

class DragDetails {
  public position: (Float, Float);
  public delta: (Float, Float);
  public velocity: (Float, Float);
}

// Example
fn DraggableBox(): Widget {
  let position = signal((0.0, 0.0));
  
  return GestureDetector {
    onDragUpdate: (details) => {
      state {
        position.value = (
          position.value.0 + details.delta.0,
          position.value.1 + details.delta.1
        );
      }
    },
    
    Container {
      transform: Transform.translate(position.value.0, position.value.1),
      color: Colors.blue,
      width: 100,
      height: 100
    }
  };
}
```

---

## 6.10 Context API

```vela
class ContextKey<T> {
  private name: String;
  
  public ContextKey(name: String) {
    this.name = name;
  }
}

// Define context keys
let ThemeContext = ContextKey<Theme>("theme");
let UserContext = ContextKey<User>("user");

// Provider
fn App(): Widget {
  let theme = Theme.dark();
  let user = getCurrentUser();
  
  return ContextProvider {
    providers: {
      ThemeContext: theme,
      UserContext: user
    },
    
    child: MainScreen()
  };
}

// Consumer
fn ThemedButton(): Widget {
  let theme = useContext(ThemeContext);
  
  return Button {
    label: "Click me",
    color: theme.primaryColor
  };
}
```

---

**FIN DEL DOCUMENTO: UI Declarativa**

Este documento define completamente el sistema de UI de Vela.
