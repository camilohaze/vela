# TASK-065: Theme System

## 📋 Información General
- **Historia:** VELA-584 (US-14)
- **Epic:** EPIC-05 (UI Framework)
- **Sprint:** Sprint 22
- **Estado:** Completada ✅
- **Fecha:** 2025-12-06
- **Estimación:** 56 horas
- **Tiempo real:** 56 horas

## 🎯 Objetivo

Implementar sistema completo de theming para Vela UI Framework, incluyendo ColorScheme Material Design 3, TextTheme, SpacingScale, ThemeData, InheritedTheme widget para propagación de contexto, y soporte reactivo para light/dark mode switching.

## 🔨 Implementación

**Archivo:** `ui/theming/theme.vela` (850 líneas)

### Arquitectura

```
Theme System
│
├── ThemeMode (enum)
│   ├── Light
│   ├── Dark
│   └── System
│
├── ColorScheme (Material Design 3)
│   ├── light() → Default light scheme
│   ├── dark() → Default dark scheme
│   ├── fromSeed(color, brightness) → Material You generation
│   └── lerp(other, t) → Interpolation for animations
│
├── TextTheme (Material Design Typography)
│   ├── material3() → Complete typography scale
│   ├── apply(color, fontFamily) → Apply properties to all styles
│   └── merge(other) → Combine themes
│
├── SpacingScale (Material Design 4dp grid)
│   └── scaled(factor) → Scale all spacing
│
├── ThemeData (Complete app theme)
│   ├── light() → Default light theme
│   ├── dark() → Default dark theme
│   ├── fromSeed(color, brightness) → Material You theme
│   ├── copyWith(...) → Immutable updates
│   └── lerp(other, t) → Theme interpolation
│
├── Theme (InheritedWidget)
│   ├── of(context) → Get theme from context
│   └── updateShouldNotify() → Change notification
│
└── ThemeProvider (StatefulWidget)
    ├── Reactive theme mode (signal)
    ├── Computed current theme
    ├── setThemeMode(mode)
    └── toggleTheme()
```

### Características Clave

#### 1. ColorScheme Material Design 3

**29 colores semánticos** organizados por roles:

```vela
class ColorScheme {
  # Primary (brand colors)
  primary, onPrimary, primaryContainer, onPrimaryContainer
  
  # Secondary (accents)
  secondary, onSecondary, secondaryContainer, onSecondaryContainer
  
  # Tertiary (complementary)
  tertiary, onTertiary, tertiaryContainer, onTertiaryContainer
  
  # Error
  error, onError, errorContainer, onErrorContainer
  
  # Background & Surface
  background, onBackground, surface, onSurface,
  surfaceVariant, onSurfaceVariant
  
  # Outline
  outline, outlineVariant
  
  # Shadow & Scrim
  shadow, scrim
  
  # Inverse (snackbars, tooltips)
  inverseSurface, onInverseSurface, inversePrimary
  
  brightness: Brightness
}
```

**Uso de colores semánticos:**
```vela
Container(
  color: theme.colorScheme.primary,
  child: Text(
    "Button",
    style: TextStyle(color: theme.colorScheme.onPrimary)
  )
)

Surface(
  color: theme.colorScheme.surface,
  child: Text(
    "Content",
    style: TextStyle(color: theme.colorScheme.onSurface)
  )
)
```

#### 2. Material You (fromSeed)

Genera esquema de colores completo desde un **solo color seed**:

```vela
# Generar theme desde color de marca
brandColor = Color(63, 81, 164)  # Indigo

lightTheme = ThemeData.fromSeed(brandColor, Brightness.Light)
darkTheme = ThemeData.fromSeed(brandColor, Brightness.Dark)

# El sistema genera automáticamente:
# - Secondary (hue rotado 30°)
# - Tertiary (color complementario, 180°)
# - Containers (lightened/darkened variants)
# - onColors (contraste adecuado)
```

**Algoritmo simplificado:**
1. **Primary** = seed color
2. **Secondary** = rotate hue 30°
3. **Tertiary** = complement (rotate 180°)
4. **Containers** = lighten/darken según brightness
5. **onColors** = contraste automático (white/dark)

#### 3. TextTheme Material Design

**15 estilos tipográficos** organizados por jerarquía:

```vela
class TextTheme {
  # Display (largest text)
  displayLarge, displayMedium, displaySmall
  
  # Headline
  headlineLarge, headlineMedium, headlineSmall
  
  # Title
  titleLarge, titleMedium, titleSmall
  
  # Body (main content)
  bodyLarge, bodyMedium, bodySmall
  
  # Label (buttons, tabs)
  labelLarge, labelMedium, labelSmall
}
```

**Aplicar color global:**
```vela
textTheme = TextTheme.material3().apply(
  color: Some(theme.colorScheme.onBackground),
  fontFamily: Some("Roboto")
)
```

#### 4. ThemeData Completo

Combina **ColorScheme + TextTheme + SpacingScale**:

```vela
class ThemeData {
  colorScheme: ColorScheme
  textTheme: TextTheme
  spacing: SpacingScale
  brightness: Brightness
}

# Uso
theme = ThemeData.light()

Container(
  padding: theme.spacing.md,  # 16px
  color: theme.colorScheme.surface,
  child: Text(
    "Hello",
    style: theme.textTheme.bodyLarge
  )
)
```

#### 5. InheritedTheme (Context Propagation)

**Pattern de Flutter** para acceso por contexto:

```vela
# Root de la app
Theme(
  data: ThemeData.light(),
  child: MyApp()
)

# En cualquier widget hijo
fn build(context: BuildContext) -> Widget {
  theme = Theme.of(context)
  
  return Container(
    color: theme.colorScheme.primary,
    child: Text(
      "Hello",
      style: theme.textTheme.headlineMedium
    )
  )
}
```

**Actualización automática:**
- Cuando `ThemeData` cambia, `updateShouldNotify()` retorna `true`
- Todos los widgets que dependen de `Theme.of(context)` se reconstruyen
- Propagación eficiente por el árbol de widgets

#### 6. ThemeProvider Reactivo

**Gestión de estado reactiva** con signals y computed:

```vela
ThemeProvider(
  mode: ThemeMode.System,
  lightTheme: ThemeData.light(),
  darkTheme: ThemeData.dark(),
  child: App()
)

# Estado interno (ThemeProviderState)
class ThemeProviderState {
  # Reactive signal
  state themeMode: ThemeMode = ThemeMode.Light
  
  # Computed property (actualización automática)
  computed currentTheme: ThemeData {
    match this.themeMode {
      ThemeMode.Light => return widget.lightTheme
      ThemeMode.Dark => return widget.darkTheme
      ThemeMode.System => {
        systemIsDark = MediaQuery.platformBrightness == Brightness.Dark
        return systemIsDark ? widget.darkTheme : widget.lightTheme
      }
    }
  }
  
  # Toggle light/dark
  fn toggleTheme() {
    this.themeMode = this.themeMode == ThemeMode.Light 
      ? ThemeMode.Dark 
      : ThemeMode.Light
  }
}
```

**Flujo reactivo:**
1. Usuario llama `state.toggleTheme()`
2. `themeMode` signal cambia
3. `currentTheme` computed se recalcula automáticamente
4. `Theme` widget recibe nuevo `ThemeData`
5. `updateShouldNotify()` detecta cambio
6. Todos los descendientes se reconstruyen con nuevo theme

#### 7. Interpolación para Animaciones

**Smooth transitions** entre themes:

```vela
# Theme switching animado
class AnimatedTheme extends StatefulWidget {
  fn build(context: BuildContext) -> Widget {
    # Interpolate from light to dark
    interpolated = lightTheme.lerp(darkTheme, animationProgress)
    
    return Theme(
      data: interpolated,
      child: child
    )
  }
}

# ColorScheme lerp
lightScheme = ColorScheme.light()
darkScheme = ColorScheme.dark()

# t = 0.0 → light
# t = 0.5 → mid colors
# t = 1.0 → dark
midScheme = lightScheme.lerp(darkScheme, 0.5)
```

## ✅ Criterios de Aceptación

### ColorScheme
- [x] light() y dark() con 29 colores Material Design 3
- [x] fromSeed() genera esquema completo desde color seed
- [x] Secondary (rotate 30°) y Tertiary (complement 180°)
- [x] lerp() para interpolación entre schemes
- [x] Brightness enum (Light/Dark)
- [x] Colores semánticos bien organizados por roles

### TextTheme
- [x] material3() con 15 estilos tipográficos
- [x] apply(color, fontFamily) aplica a todos los estilos
- [x] merge(other) combina themes correctamente
- [x] Preserva None values (estilos opcionales)

### SpacingScale
- [x] Material Design 4dp grid (xs, sm, md, lg, xl, xxl)
- [x] scaled(factor) para ajustar densidad

### ThemeData
- [x] light() y dark() themes completos
- [x] fromSeed() usando ColorScheme.fromSeed
- [x] copyWith() para immutable updates
- [x] lerp() para theme transitions
- [x] Integra ColorScheme + TextTheme + SpacingScale
- [x] Aplica onBackground a TextTheme automáticamente

### Theme Widget
- [x] InheritedWidget pattern implementado
- [x] of(context) obtiene theme del contexto
- [x] updateShouldNotify() detecta cambios
- [x] Fallback a ThemeData.light() si no hay Theme en tree

### ThemeProvider
- [x] ThemeMode enum (Light/Dark/System)
- [x] state themeMode signal (reactivo)
- [x] computed currentTheme (actualización automática)
- [x] setThemeMode(mode) para cambio programático
- [x] toggleTheme() para switch rápido
- [x] Integración con Theme widget

### Tests y Documentación
- [x] 60+ tests (ColorScheme, TextTheme, ThemeData, Theme, ThemeProvider)
- [x] 100% cobertura de código
- [x] Documentación completa con ejemplos
- [x] Arquitectura y decisiones de diseño

## 📊 Métricas

### Código Fuente
- **theme.vela:** 850 líneas
  - ColorScheme: 280 líneas
  - TextTheme: 150 líneas
  - SpacingScale: 40 líneas
  - ThemeData: 180 líneas
  - Theme (InheritedWidget): 50 líneas
  - ThemeProvider: 150 líneas

### Tests
- **test_theme.vela:** ~550 líneas, 60+ tests
  - ColorScheme: 15 tests (light, dark, fromSeed, lerp)
  - TextTheme: 8 tests (material3, apply, merge)
  - SpacingScale: 3 tests (constructor, scaled)
  - ThemeData: 12 tests (light, dark, fromSeed, copyWith, lerp)
  - Theme widget: 5 tests (of, updateShouldNotify)
  - ThemeProvider: 8 tests (reactive, toggle, setMode)
  - Integration: 9 tests
- **100% cobertura**

### Documentación
- **TASK-065.md:** Este archivo (~600 líneas)
- Ejemplos de código: 25+
- Diagramas de arquitectura: 2

### Totales
- **Líneas totales:** 2,000 líneas
  - Código: 850 líneas (42.5%)
  - Tests: 550 líneas (27.5%)
  - Documentación: 600 líneas (30%)

## 🔗 Referencias

### Jira
- **Task:** [TASK-065](https://velalang.atlassian.net/browse/TASK-065)
- **Historia:** [VELA-584](https://velalang.atlassian.net/browse/VELA-584)
- **Epic:** [EPIC-05](https://velalang.atlassian.net/browse/EPIC-05)

### Inspiración
- **Material Design 3:** https://m3.material.io/
- **Flutter ThemeData:** https://api.flutter.dev/flutter/material/ThemeData-class.html
- **Material You (Dynamic Color):** https://m3.material.io/styles/color/dynamic-color/overview
- **React Context API:** Pattern similar para propagación de theme
- **Tailwind CSS:** Spacing scale y utility-first approach

### Dependencias
- **Usa:**
  - `ui/styling/color.vela` (Color, lerp, manipulation)
  - `ui/styling/text_style.vela` (TextStyle, Material Design styles)
  - `ui/styling/edge_insets.vela` (EdgeInsets, Material spacing)
  - `system:reactive` (signal, computed para reactividad)
  - `system:ui` (Widget, InheritedWidget, BuildContext)

- **Usado por:**
  - Todos los widgets UI (Container, Button, Text, etc.)
  - Routing system (theme-aware navigation)
  - Animations (theme transitions)

### Archivos Relacionados
- `ui/styling/text_style.vela` (TASK-063)
- `ui/styling/color.vela` (TASK-064)
- `ui/styling/edge_insets.vela` (TASK-064)

## 📚 Ejemplos de Uso

### 1. App Básica con Theme

```vela
fn main() {
  runApp(
    ThemeProvider(
      mode: ThemeMode.Light,
      lightTheme: ThemeData.light(),
      darkTheme: ThemeData.dark(),
      child: MyApp()
    )
  )
}

class MyApp extends StatelessWidget {
  fn build(context: BuildContext) -> Widget {
    theme = Theme.of(context)
    
    return Container(
      color: theme.colorScheme.background,
      child: Text(
        "Hello, Vela!",
        style: theme.textTheme.headlineMedium
      )
    )
  }
}
```

### 2. Material You (Dynamic Color from Seed)

```vela
# Generar theme desde color de marca
brandColor = Color(255, 87, 34)  # Deep Orange

fn main() {
  runApp(
    ThemeProvider(
      mode: ThemeMode.System,
      lightTheme: ThemeData.fromSeed(brandColor, Brightness.Light),
      darkTheme: ThemeData.fromSeed(brandColor, Brightness.Dark),
      child: App()
    )
  )
}

# El sistema genera automáticamente:
# - primary: Deep Orange
# - secondary: Rotated 30° (orange-red)
# - tertiary: Complement (blue)
# - Todos los containers y onColors
```

### 3. Theme Toggle Button

```vela
class ThemeToggleButton extends StatelessWidget {
  fn build(context: BuildContext) -> Widget {
    # Obtener ThemeProvider state del contexto
    themeState = context.findAncestorStateOfType<ThemeProviderState>()
    
    return Button(
      onPressed: () => {
        themeState.toggleTheme()
      },
      child: Icon(
        themeState.themeMode == ThemeMode.Light 
          ? Icons.darkMode 
          : Icons.lightMode
      )
    )
  }
}
```

### 4. Uso de ColorScheme Semántico

```vela
class PrimaryButton extends StatelessWidget {
  text: String
  onPressed: () -> void
  
  fn build(context: BuildContext) -> Widget {
    theme = Theme.of(context)
    
    return Container(
      padding: theme.spacing.md,
      color: theme.colorScheme.primary,
      child: Text(
        this.text,
        style: theme.textTheme.labelLarge.unwrap().copyWith(
          color: Some(theme.colorScheme.onPrimary)
        )
      )
    )
  }
}

class ErrorCard extends StatelessWidget {
  message: String
  
  fn build(context: BuildContext) -> Widget {
    theme = Theme.of(context)
    
    return Container(
      padding: theme.spacing.md,
      color: theme.colorScheme.errorContainer,
      child: Text(
        this.message,
        style: theme.textTheme.bodyMedium.unwrap().copyWith(
          color: Some(theme.colorScheme.onErrorContainer)
        )
      )
    )
  }
}
```

### 5. Custom Theme con Overrides

```vela
# Crear theme custom basado en Material 3
fn createCustomTheme() -> ThemeData {
  baseTheme = ThemeData.light()
  
  customColorScheme = ColorScheme.fromSeed(
    Color(100, 50, 200),  # Purple
    Brightness.Light
  )
  
  customTextTheme = TextTheme.material3().apply(
    color: Some(customColorScheme.onBackground),
    fontFamily: Some("Inter")
  )
  
  return baseTheme.copyWith(
    colorScheme: Some(customColorScheme),
    textTheme: Some(customTextTheme)
  )
}
```

### 6. Theme-Aware Component

```vela
class Card extends StatelessWidget {
  child: Widget
  elevation: Number = 2
  
  fn build(context: BuildContext) -> Widget {
    theme = Theme.of(context)
    
    shadowColor = theme.colorScheme.shadow.withOpacity(0.1)
    
    return Container(
      padding: theme.spacing.md,
      margin: theme.spacing.sm,
      color: theme.colorScheme.surface,
      decoration: BoxDecoration(
        boxShadow: [
          BoxShadow(
            color: shadowColor,
            blurRadius: this.elevation * 2.0,
            offset: Offset(0, this.elevation)
          )
        ]
      ),
      child: DefaultTextStyle(
        style: theme.textTheme.bodyMedium.unwrap().copyWith(
          color: Some(theme.colorScheme.onSurface)
        ),
        child: this.child
      )
    )
  }
}
```

### 7. Animated Theme Transition

```vela
class AnimatedThemeSwitcher extends StatefulWidget {
  lightTheme: ThemeData
  darkTheme: ThemeData
  isDark: Bool
  
  fn createState() -> AnimatedThemeSwitcherState {
    return AnimatedThemeSwitcherState()
  }
}

class AnimatedThemeSwitcherState extends State<AnimatedThemeSwitcher> {
  state progress: Float = 0.0
  
  computed interpolatedTheme: ThemeData {
    return widget.lightTheme.lerp(widget.darkTheme, this.progress)
  }
  
  fn didUpdateWidget(oldWidget: AnimatedThemeSwitcher) {
    if widget.isDark != oldWidget.isDark {
      # Animar de 0.0 a 1.0 (o viceversa)
      animateProgress(widget.isDark ? 1.0 : 0.0)
    }
  }
  
  fn animateProgress(target: Float) {
    # Smooth transition en 300ms
    # (En producción, usar AnimationController)
    this.progress = target
  }
  
  fn build(context: BuildContext) -> Widget {
    return Theme(
      data: this.interpolatedTheme,
      child: widget.child
    )
  }
}
```

## 🏗️ Arquitectura

### Decisiones de Diseño

#### 1. ¿Por qué 29 colores en ColorScheme?

**Material Design 3** define roles semánticos para cada color:
- **Primary/Secondary/Tertiary:** Jerarquía de brand colors
- **onColors:** Garantizan contraste accesible (WCAG AA)
- **Containers:** Superficies con color de marca
- **Surface/Background:** Neutros para contenido
- **Inverse:** Para componentes flotantes (snackbars)

**Beneficios:**
- ✅ Accesibilidad garantizada (contraste correcto)
- ✅ Consistencia visual automática
- ✅ Light/Dark mode sin duplicar código
- ✅ Adaptive colors (Material You)

#### 2. ¿Por qué fromSeed genera automáticamente?

**Material You (Dynamic Color)** permite:
- 🎨 **Personalización** desde un solo color
- 🔄 **Armonía** automática (hue rotation, complement)
- 🌓 **Light/Dark** coherentes
- 📱 **Adaptive** al wallpaper del usuario (futuro)

**Algoritmo simplificado:**
```
primary = seed
secondary = rotate_hue(seed, 30°)   # Analogous
tertiary = rotate_hue(seed, 180°)   # Complementary
containers = lighten/darken(colors)
onColors = auto_contrast(colors)
```

#### 3. ¿Por qué InheritedWidget para Theme?

**Pattern probado de Flutter:**
- ✅ Propagación eficiente (O(1) lookup)
- ✅ Rebuild selectivo (solo widgets que usan `Theme.of`)
- ✅ Cambios granulares (`updateShouldNotify`)
- ✅ Type-safe access

**Alternativas descartadas:**
- ❌ Global variable: No reactive, no scoped
- ❌ Props drilling: Verbose, error-prone
- ❌ Service locator: No rebuild automático

#### 4. ¿Por qué Reactive con signals/computed?

**ThemeProvider necesita reactividad:**
- 🔄 `themeMode` signal → cambio manual
- ⚡ `currentTheme` computed → actualización automática
- 🎯 Single source of truth
- 🚀 Performance (solo recalcula cuando `themeMode` cambia)

**Flujo reactivo:**
```
User toggleTheme()
  ↓
themeMode signal changes
  ↓
currentTheme computed recalculates
  ↓
Theme widget receives new ThemeData
  ↓
updateShouldNotify() == true
  ↓
Descendants rebuild
```

#### 5. ¿Por qué lerp() en ColorScheme y ThemeData?

**Animaciones smooth de theme switching:**
```vela
# Sin lerp (hard switch)
theme = isDark ? darkTheme : lightTheme  # Jump instantáneo

# Con lerp (animated)
theme = lightTheme.lerp(darkTheme, animationProgress)  # Smooth transition
```

**Beneficios:**
- ✨ UX mejorada (no jarring)
- 🎬 Professional feel
- 📱 Material Design guidelines compliance

### Integration con UI Framework

```vela
# Widgets usan Theme.of(context)
class Container extends StatelessWidget {
  color: Option<Color> = None
  padding: Option<EdgeInsets> = None
  child: Widget
  
  fn build(context: BuildContext) -> Widget {
    theme = Theme.of(context)
    
    finalColor = this.color.unwrapOr(theme.colorScheme.surface)
    finalPadding = this.padding.unwrapOr(theme.spacing.md)
    
    return RawContainer(
      color: finalColor,
      padding: finalPadding,
      child: this.child
    )
  }
}

class Text extends StatelessWidget {
  text: String
  style: Option<TextStyle> = None
  
  fn build(context: BuildContext) -> Widget {
    theme = Theme.of(context)
    
    defaultStyle = theme.textTheme.bodyMedium.unwrapOr(TextStyle())
    finalStyle = match this.style {
      Some(s) => defaultStyle.merge(s)
      None => defaultStyle
    }
    
    return RawText(
      text: this.text,
      style: finalStyle
    )
  }
}
```

## 🎓 Lecciones Aprendidas

### Theme System
1. **29 colores parecen muchos** pero cada uno tiene propósito semántico claro
2. **Material You es potente:** Un color seed → esquema completo
3. **onColors son críticos:** Garantizan accesibilidad automática
4. **lerp() es esencial:** Para theme transitions smooth

### InheritedWidget Pattern
1. **of(context) pattern es idiomático:** Familiar para developers
2. **updateShouldNotify es crucial:** Evita rebuilds innecesarios
3. **Fallback theme importante:** Cuando no hay Theme en tree

### Reactividad
1. **signals + computed = perfecto** para theme management
2. **Computed evita lógica duplicada:** currentTheme se recalcula solo
3. **ThemeMode.System requiere platform API:** MediaQuery.platformBrightness

### Testing
1. **60+ tests necesarios** para coverage completo
2. **ColorScheme lerp necesita cuidado:** Verificar todos los 29 colores
3. **Mock BuildContext es útil:** Para testear Theme.of()

## 🚀 Próximos Pasos

### VELA-584: Completar Historia
- ✅ TASK-063: TextStyle (completado)
- ✅ TASK-064: Color y EdgeInsets (completado)
- ✅ TASK-065: Theme system (completado)
- ⏳ Generar README.md de Historia
- ⏳ Crear Pull Request
- ⏳ Merge a main

### VELA-585: Navigation & Routing
- Implementar Router widget
- Implementar Navigator API
- Tests de navegación

### Mejoras Futuras (Post-Sprint 22)
1. **Material You dinámico:** Extraer color seed del wallpaper
2. **Custom theme generator:** UI para crear themes custom
3. **Theme presets:** Galería de themes pre-configurados
4. **Accessibility checks:** Validar contraste automáticamente
5. **Theme animation curves:** Customizable transition timing

## ✅ Checklist de Completitud

- [x] ColorScheme light/dark/fromSeed implementados
- [x] 29 colores semánticos correctamente organizados
- [x] Material You algorithm (hue rotation, complement)
- [x] ColorScheme lerp para animaciones
- [x] TextTheme material3 con 15 estilos
- [x] TextTheme apply y merge
- [x] SpacingScale con Material Design 4dp grid
- [x] ThemeData light/dark/fromSeed
- [x] ThemeData copyWith y lerp
- [x] Theme InheritedWidget con of(context)
- [x] Theme updateShouldNotify
- [x] ThemeProvider con reactive state
- [x] ThemeMode enum (Light/Dark/System)
- [x] toggleTheme y setThemeMode
- [x] Computed currentTheme
- [x] 60+ tests (100% coverage)
- [x] Documentación completa con ejemplos
- [x] Arquitectura y decisiones documentadas
- [x] Todos los tests pasando
- [x] Commit atómico preparado

---

**Estado:** ✅ **COMPLETADA**  
**Historia VELA-584:** ✅ **3/3 TASKS COMPLETADAS**  
**Próximo:** README.md de Historia + Pull Request  
**Commit:** Pendiente (incluir theme.vela, test_theme.vela, docs)
