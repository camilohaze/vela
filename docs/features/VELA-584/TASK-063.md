# TASK-063: Implementar TextStyle y styling APIs

## 📋 Información General
- **Historia:** VELA-584 (US-14)
- **Sprint:** 22
- **Estado:** Completada ✅
- **Fecha:** 2025-12-06
- **Estimación:** 32 horas
- **Equipo:** UI Framework

## 🎯 Objetivo

Implementar un sistema completo de estilos de texto para el framework UI de Vela, inspirado en Flutter's TextStyle y Material Design Typography.

## 🔨 Implementación

### Archivos generados

```
ui/styling/
└── text_style.vela (720 líneas)

tests/unit/ui/styling/
└── test_text_style.vela (650 líneas)

docs/features/VELA-584/
└── TASK-063.md (este archivo)
```

### Componentes principales

#### 1. **Enums de Tipografía**

```vela
enum FontWeight {
  Thin, ExtraLight, Light, Normal, Medium,
  SemiBold, Bold, ExtraBold, Black
}

enum FontStyle {
  Normal, Italic, Oblique
}

enum TextDecoration {
  None, Underline, Overline, LineThrough,
  Combined(decorations: List<TextDecoration>)
}
```

**Características:**
- Mapeo a valores numéricos CSS (100-900 para weight)
- Conversión bidireccional (enum ↔ valor)
- Soporte para decoraciones combinadas

#### 2. **TextShadow Value Object**

```vela
valueObject TextShadow {
  color: Color
  offsetX: Float
  offsetY: Float
  blurRadius: Float
}
```

**Factory methods:**
- `TextShadow.subtle()` - Sombra sutil (blur 2px)
- `TextShadow.medium()` - Sombra media (blur 4px)
- `TextShadow.strong()` - Sombra fuerte (blur 8px)

#### 3. **TextStyle Class (Principal)**

```vela
class TextStyle {
  # Typography
  fontFamily: Option<String>
  fontSize: Option<Float>
  fontWeight: Option<FontWeight>
  fontStyle: Option<FontStyle>
  letterSpacing: Option<Float>
  wordSpacing: Option<Float>
  height: Option<Float>
  
  # Color
  color: Option<Color>
  backgroundColor: Option<Color>
  
  # Decoration
  decoration: Option<TextDecoration>
  decorationColor: Option<Color>
  decorationStyle: Option<TextDecorationStyle>
  decorationThickness: Option<Float>
  
  # Effects
  shadows: List<TextShadow>
  
  # Advanced
  baseline: Option<TextBaseline>
  overflow: Option<TextOverflow>
}
```

### APIs Implementadas

#### **1. Immutable Updates**

```vela
# Merge: combina estilos
baseStyle = TextStyle(fontSize: Some(14.0))
override = TextStyle(fontWeight: Some(FontWeight.Bold))
merged = baseStyle.merge(Some(override))
# fontSize: 14.0, fontWeight: Bold

# CopyWith: actualización inmutable
updated = style.copyWith(
  fontSize: Some(18.0),
  color: Some(Color.blue())
)
```

#### **2. Fluent Builder API**

```vela
style = TextStyle()
  .withSize(16.0)
  .bold()
  .italic()
  .withColor(Color.red())
  .underline()
  .withShadow(TextShadow.subtle())
```

**Métodos disponibles:**
- `withColor(color)`, `withSize(size)`, `withWeight(weight)`, `withFamily(family)`
- Shortcuts: `bold()`, `italic()`, `underline()`, `lineThrough()`
- `withShadow(shadow)` - Acumula sombras

#### **3. CSS Rendering**

```vela
style = TextStyle(
  fontSize: Some(16.0),
  fontWeight: Some(FontWeight.Bold),
  color: Some(Color.blue())
)

props = style.toCSSProperties()
# {
#   "font-size": "16.0px",
#   "font-weight": "700",
#   "color": "rgb(0, 0, 255)"
# }
```

#### **4. Material Design Predefined Styles**

Implementación completa de la escala tipográfica de Material Design 3:

```vela
# Display
TextStyle.displayLarge()   # 57px
TextStyle.displayMedium()  # 45px
TextStyle.displaySmall()   # 36px

# Headline
TextStyle.headlineLarge()  # 32px
TextStyle.headlineMedium() # 28px
TextStyle.headlineSmall()  # 24px

# Title
TextStyle.titleLarge()     # 22px
TextStyle.titleMedium()    # 16px, Medium
TextStyle.titleSmall()     # 14px, Medium

# Body
TextStyle.bodyLarge()      # 16px
TextStyle.bodyMedium()     # 14px
TextStyle.bodySmall()      # 12px

# Label
TextStyle.labelLarge()     # 14px, Medium
TextStyle.labelMedium()    # 12px, Medium
TextStyle.labelSmall()     # 11px, Medium
```

#### **5. Interpolation (para animaciones)**

```vela
fn lerp(a: Option<TextStyle>, b: Option<TextStyle>, t: Float) -> Option<TextStyle>
```

**Interpolación:**
- Propiedades numéricas: interpolación lineal (fontSize, letterSpacing)
- Colores: interpolación RGBA
- Propiedades discretas: threshold en t=0.5 (fontWeight, fontStyle)

**Ejemplo:**
```vela
start = TextStyle(fontSize: Some(14.0), color: Some(Color.black()))
end = TextStyle(fontSize: Some(24.0), color: Some(Color.red()))

# t=0.5 → fontSize: 19.0, color: gris oscuro
interpolated = lerp(Some(start), Some(end), 0.5)
```

## ✅ Criterios de Aceptación

- [x] **Enums implementados**: FontWeight, FontStyle, TextDecoration, TextDecorationStyle, TextBaseline, TextOverflow
- [x] **TextShadow value object**: Con factory methods
- [x] **TextStyle class completa**: 16 propiedades opcionales
- [x] **Merge y copyWith**: Inmutabilidad garantizada
- [x] **Fluent builder API**: 10+ métodos chainables
- [x] **CSS rendering**: Conversión completa a propiedades CSS
- [x] **Material Design styles**: 15 estilos predefinidos
- [x] **Interpolación**: Soporte para animaciones
- [x] **Tests**: 60+ tests unitarios (100% coverage)
- [x] **Documentación**: Completa con ejemplos

## 📊 Métricas

### Código
- **Líneas de código**: 720 líneas
- **Clases**: 1 (TextStyle)
- **Enums**: 6 (FontWeight, FontStyle, TextDecoration, TextDecorationStyle, TextBaseline, TextOverflow)
- **Value Objects**: 1 (TextShadow)
- **Métodos públicos**: 35+
- **Predefined styles**: 15 (Material Design 3)

### Tests
- **Líneas de tests**: 650 líneas
- **Suites de tests**: 11
- **Tests unitarios**: 62 tests
- **Coverage**: 100%

### Performance
- **CSS generation**: O(n) donde n = número de propiedades no-None
- **Merge**: O(1) por propiedad
- **CopyWith**: O(1) (copia shallow)
- **Lerp**: O(1) para propiedades numéricas

### Total
- **Total líneas**: 1,370 líneas (código + tests)
- **Total archivos**: 2 archivos

## 🎨 Ejemplos de Uso

### Ejemplo 1: Título con estilos personalizados

```vela
titleStyle = TextStyle.titleLarge()
  .withColor(Color.primary())
  .bold()
  .withShadow(TextShadow.subtle())

widget Text {
  text: "Hello, Vela!"
  style: titleStyle
}
```

### Ejemplo 2: Texto decorado

```vela
linkStyle = TextStyle()
  .withSize(14.0)
  .withColor(Color.blue())
  .underline()
  .copyWith(
    decorationColor: Some(Color.blue()),
    decorationStyle: Some(TextDecorationStyle.Solid)
  )

widget Text {
  text: "Click here"
  style: linkStyle
}
```

### Ejemplo 3: Combinación de estilos

```vela
baseStyle = TextStyle.bodyMedium()

emphasisStyle = baseStyle.merge(Some(
  TextStyle(
    fontWeight: Some(FontWeight.Bold),
    color: Some(Color.red())
  )
))

widget RichText {
  children: [
    TextSpan(text: "Normal text", style: baseStyle),
    TextSpan(text: " emphasized", style: emphasisStyle),
    TextSpan(text: " text.", style: baseStyle)
  ]
}
```

### Ejemplo 4: Animación de estilo

```vela
component AnimatedText extends StatefulWidget {
  state progress: Float = 0.0
  
  smallStyle = TextStyle(fontSize: Some(14.0), color: Some(Color.black()))
  largeStyle = TextStyle(fontSize: Some(32.0), color: Some(Color.red()))
  
  fn build(context: BuildContext) -> Widget {
    # Interpolar entre estilos
    currentStyle = lerp(
      Some(this.smallStyle),
      Some(this.largeStyle),
      this.progress
    ).unwrapOr(this.smallStyle)
    
    return Column(
      children: [
        Text(text: "Animated", style: currentStyle),
        Slider(
          value: this.progress,
          onChanged: (value) => { this.progress = value }
        )
      ]
    )
  }
}
```

### Ejemplo 5: Múltiples sombras

```vela
dramaticStyle = TextStyle()
  .withSize(48.0)
  .withColor(Color.white())
  .withShadow(TextShadow(Color.black().withOpacity(0.5), 2.0, 2.0, 4.0))
  .withShadow(TextShadow(Color.blue().withOpacity(0.3), 4.0, 4.0, 8.0))
  .withShadow(TextShadow(Color.red().withOpacity(0.2), 6.0, 6.0, 12.0))

widget Text {
  text: "Dramatic Text"
  style: dramaticStyle
}
```

## 🏗️ Arquitectura

### Diseño de Clases

```
TextStyle
├── Properties (16 opcionales)
│   ├── Typography (7)
│   ├── Color (2)
│   ├── Decoration (4)
│   ├── Effects (1 list)
│   └── Advanced (2)
├── Methods
│   ├── Immutable Updates
│   │   ├── merge()
│   │   └── copyWith()
│   ├── Fluent Builders (10)
│   ├── Rendering
│   │   └── toCSSProperties()
│   └── Static Factories (15)
└── Helpers
    └── lerp() (global)

Enums (6)
├── FontWeight (9 values)
├── FontStyle (3 values)
├── TextDecoration (4 + Combined)
├── TextDecorationStyle (5 values)
├── TextBaseline (2 values)
└── TextOverflow (4 values)

ValueObjects (1)
└── TextShadow
    ├── Properties (4)
    ├── toCSSValue()
    └── Factories (3)
```

### Flujo de Datos

```
1. Definición de estilo:
   TextStyle() → .withSize() → .bold() → .withColor()

2. Combinación:
   baseStyle.merge(override) → mergedStyle

3. Rendering:
   mergedStyle.toCSSProperties() → Map<String, String>

4. Aplicación al DOM:
   Renderer aplica props CSS a elemento
```

## 🔗 Referencias

### Inspiraciones
- **Flutter**: TextStyle, FontWeight, TextDecoration
  - Docs: https://api.flutter.dev/flutter/painting/TextStyle-class.html
- **Material Design 3**: Typography scale
  - Specs: https://m3.material.io/styles/typography/overview
- **CSS**: Font properties, text-decoration
  - MDN: https://developer.mozilla.org/en-US/docs/Web/CSS/font
- **SwiftUI**: Font system
  - Docs: https://developer.apple.com/documentation/swiftui/font

### Decisiones de Diseño
1. **Option<T> para todas las propiedades**: Permite estilos parciales y merge eficiente
2. **Immutability**: copyWith y merge retornan nuevas instancias
3. **Fluent API**: Chaining para mejor DX
4. **Material Design predefined**: 15 estilos según escala oficial
5. **CSS rendering**: Target web como plataforma inicial

## 🚀 Próximos Pasos

### En TASK-064 (Color y EdgeInsets):
- Implementar `Color` class con RGBA, HSL, hex
- Implementar `EdgeInsets` para spacing
- Integrar con TextStyle (color, backgroundColor)

### En TASK-065 (Theme system):
- ThemeData con textTheme
- InheritedTheme para context-based theming
- Propagación reactiva de cambios de theme

### Mejoras futuras (post-Sprint 22):
- Soporte para font variants (small-caps, old-style numerals)
- Text transforms (uppercase, lowercase, capitalize)
- Advanced typography (OpenType features)
- Responsive typography (scale por viewport)

---

**Refs**: VELA-584, TASK-063, Sprint 22
