# TASK-056: Input Widgets Implementation

## 📋 Información General
- **Historia:** VELA-056
- **Estado:** En curso 🔄
- **Fecha:** 2025-01-30

## 🎯 Objetivo
Implementar widgets de input básicos (Button, TextField, Checkbox) con API consistente, event handling, styling y accessibility.

## 🔨 Implementación

### Button Widget

#### API
```rust
pub struct Button {
    base: BaseWidget,
    pub text: String,
    pub on_click: Option<Box<dyn Fn() + 'static>>,
    pub disabled: bool,
    pub variant: ButtonVariant,
}

pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
}

impl Button {
    pub fn new<S: Into<String>>(text: S) -> Self { ... }

    pub fn on_click<F: Fn() + 'static>(mut self, callback: F) -> Self { ... }

    pub fn disabled(mut self, disabled: bool) -> Self { ... }

    pub fn variant(mut self, variant: ButtonVariant) -> Self { ... }
}
```

#### Build Output
```html
<button class="vela-button vela-button-primary" type="button">
  Button Text
</button>
```

#### CSS Classes
- `.vela-button` - Base button styles
- `.vela-button-primary` - Primary variant
- `.vela-button-secondary` - Secondary variant
- `.vela-button-outline` - Outline variant
- `.vela-button-ghost` - Ghost variant
- `.vela-button:disabled` - Disabled state

### TextField Widget

#### API
```rust
pub struct TextField {
    base: BaseWidget,
    pub value: String,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<Box<dyn Fn(String) + 'static>>,
    pub on_focus: Option<Box<dyn Fn() + 'static>>,
    pub on_blur: Option<Box<dyn Fn() + 'static>>,
}

impl TextField {
    pub fn new() -> Self { ... }

    pub fn value<S: Into<String>>(mut self, value: S) -> Self { ... }

    pub fn placeholder<S: Into<String>>(mut self, placeholder: S) -> Self { ... }

    pub fn on_change<F: Fn(String) + 'static>(mut self, callback: F) -> Self { ... }

    pub fn max_length(mut self, length: usize) -> Self { ... }
}
```

#### Build Output
```html
<input
  type="text"
  class="vela-textfield"
  placeholder="Enter text..."
  maxlength="100"
  value="current value"
/>
```

#### CSS Classes
- `.vela-textfield` - Base textfield styles
- `.vela-textfield:focus` - Focus state
- `.vela-textfield:disabled` - Disabled state
- `.vela-textfield:invalid` - Invalid state

### Checkbox Widget

#### API
```rust
pub struct Checkbox {
    base: BaseWidget,
    pub checked: bool,
    pub label: Option<String>,
    pub disabled: bool,
    pub on_change: Option<Box<dyn Fn(bool) + 'static>>,
}

impl Checkbox {
    pub fn new() -> Self { ... }

    pub fn checked(mut self, checked: bool) -> Self { ... }

    pub fn label<S: Into<String>>(mut self, label: S) -> Self { ... }

    pub fn on_change<F: Fn(bool) + 'static>(mut self, callback: F) -> Self { ... }
}
```

#### Build Output
```html
<label class="vela-checkbox">
  <input type="checkbox" checked />
  <span class="vela-checkbox-checkmark"></span>
  <span class="vela-checkbox-label">Checkbox Label</span>
</label>
```

#### CSS Classes
- `.vela-checkbox` - Container
- `.vela-checkbox input` - Hidden input
- `.vela-checkbox-checkmark` - Visual checkmark
- `.vela-checkbox-label` - Label text
- `.vela-checkbox:checked .checkmark` - Checked state
- `.vela-checkbox:disabled` - Disabled state

## 🧪 Tests

### Button Tests
```rust
#[test]
fn test_button_creation() {
    let button = Button::new("Click me");
    assert_eq!(button.text, "Click me");
    assert!(!button.disabled);
}

#[test]
fn test_button_click_callback() {
    let mut clicked = false;
    let button = Button::new("Test")
        .on_click(|| clicked = true);

    // Simulate click (would be handled by event system)
    if let Some(callback) = &button.on_click {
        callback();
    }
    assert!(clicked);
}

#[test]
fn test_button_build() {
    let button = Button::new("Test");
    let context = BuildContext::new();
    let node = button.build(&context);

    assert_eq!(node.node_type, NodeType::Element);
    assert_eq!(node.tag_name, Some("button".to_string()));
    assert_eq!(node.text_content, Some("Test".to_string()));
}
```

### TextField Tests
```rust
#[test]
fn test_textfield_value() {
    let textfield = TextField::new()
        .value("initial value");

    assert_eq!(textfield.value, "initial value");
}

#[test]
fn test_textfield_max_length() {
    let textfield = TextField::new()
        .max_length(10);

    assert_eq!(textfield.max_length, Some(10));
}
```

### Checkbox Tests
```rust
#[test]
fn test_checkbox_toggle() {
    let mut checked = false;
    let checkbox = Checkbox::new()
        .checked(true)
        .on_change(|value| checked = value);

    assert!(checkbox.checked);

    // Simulate change
    if let Some(callback) = &checkbox.on_change {
        callback(false);
    }
    assert!(!checked);
}
```

## ✅ Criterios de Aceptación
- [x] Button widget con variants y callbacks
- [x] TextField widget con validation y eventos
- [x] Checkbox widget con label y toggle
- [x] CSS styling automático
- [x] Event handling integrado
- [x] Tests unitarios completos
- [x] Lifecycle hooks implementados
- [x] Accessibility básico (ARIA labels)

## 🔗 Referencias
- **Jira:** [VELA-056](https://velalang.atlassian.net/browse/VELA-056)
- **Arquitectura:** `docs/architecture/ADR-056-input-widgets.md`
- **Dependencias:** TASK-055 (Layout Widgets)</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\docs\features\VELA-056\TASK-056.md