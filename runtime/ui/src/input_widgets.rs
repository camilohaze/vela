//! Input Widgets Implementation
//!
//! This module contains the implementation of input widgets:
//! - Button: Clickable buttons with different variants
//! - TextField: Text input fields with validation
//! - Checkbox: Boolean toggle inputs

use crate::vdom::VDomNode;
use crate::context::BuildContext;
use crate::layout::{BoxConstraints, Size};
use std::rc::Rc;
use std::cell::RefCell;

/// Button variants for different visual styles
#[derive(Debug, Clone, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Outline,
    Ghost,
}

/// Button widget for user interactions
#[derive(Clone)]
pub struct Button {
    pub text: String,
    pub variant: ButtonVariant,
    pub disabled: bool,
    pub on_click: Option<Rc<RefCell<dyn FnMut()>>>,
}

impl std::fmt::Debug for Button {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Button")
            .field("text", &self.text)
            .field("variant", &self.variant)
            .field("disabled", &self.disabled)
            .field("on_click", &self.on_click.is_some())
            .finish()
    }
}

impl Button {
    /// Create a new button with the given text
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            variant: ButtonVariant::Primary,
            disabled: false,
            on_click: None,
        }
    }

    /// Set the button variant
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set the disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the click handler
    pub fn on_click<F>(mut self, callback: F) -> Self
    where
        F: FnMut() + 'static,
    {
        self.on_click = Some(Rc::new(RefCell::new(callback)));
        self
    }

    /// Generate CSS for the button
    pub fn generate_css(&self) -> String {
        let base_css = r#"
            .button {
                padding: 8px 16px;
                border: 1px solid transparent;
                border-radius: 4px;
                font-size: 14px;
                font-weight: 500;
                cursor: pointer;
                transition: all 0.2s ease;
                display: inline-block;
                text-align: center;
                text-decoration: none;
                user-select: none;
            }
            .button:disabled {
                cursor: not-allowed;
                opacity: 0.6;
            }
        "#;

        let variant_css = match self.variant {
            ButtonVariant::Primary => r#"
                .button-primary {
                    background-color: #007bff;
                    color: white;
                    border-color: #007bff;
                }
                .button-primary:hover:not(:disabled) {
                    background-color: #0056b3;
                    border-color: #0056b3;
                }
            "#,
            ButtonVariant::Secondary => r#"
                .button-secondary {
                    background-color: #6c757d;
                    color: white;
                    border-color: #6c757d;
                }
                .button-secondary:hover:not(:disabled) {
                    background-color: #545b62;
                    border-color: #545b62;
                }
            "#,
            ButtonVariant::Outline => r#"
                .button-outline {
                    background-color: transparent;
                    color: #007bff;
                    border-color: #007bff;
                }
                .button-outline:hover:not(:disabled) {
                    background-color: #007bff;
                    color: white;
                }
            "#,
            ButtonVariant::Ghost => r#"
                .button-ghost {
                    background-color: transparent;
                    color: #007bff;
                    border-color: transparent;
                }
                .button-ghost:hover:not(:disabled) {
                    background-color: rgba(0, 123, 255, 0.1);
                }
            "#,
        };

        format!("{}{}", base_css, variant_css)
    }
}

impl crate::widget::Widget for Button {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("type".to_string(), "button".to_string());

        if self.disabled {
            attributes.insert("disabled".to_string(), "true".to_string());
        }

        let class_name = match self.variant {
            ButtonVariant::Primary => "button button-primary",
            ButtonVariant::Secondary => "button button-secondary",
            ButtonVariant::Outline => "button button-outline",
            ButtonVariant::Ghost => "button button-ghost",
        };
        attributes.insert("class".to_string(), class_name.to_string());

        VDomNode {
            node_type: crate::vdom::NodeType::Element,
            tag_name: Some("button".to_string()),
            attributes,
            children: Vec::new(),
            text_content: Some(self.text.clone()),
            event_listeners: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            key: None,
        }
    }
}

/// TextField widget for text input
#[derive(Clone)]
pub struct TextField {
    pub value: String,
    pub placeholder: String,
    pub disabled: bool,
    pub max_length: Option<usize>,
    pub on_change: Option<Rc<RefCell<dyn FnMut(String)>>>,
}

impl std::fmt::Debug for TextField {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextField")
            .field("value", &self.value)
            .field("placeholder", &self.placeholder)
            .field("disabled", &self.disabled)
            .field("max_length", &self.max_length)
            .field("on_change", &self.on_change.is_some())
            .finish()
    }
}

impl TextField {
    /// Create a new text field
    pub fn new() -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            disabled: false,
            max_length: None,
            on_change: None,
        }
    }

    /// Set the initial value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    /// Set the placeholder text
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Set the disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the maximum length
    pub fn max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }

    /// Set the change handler
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: FnMut(String) + 'static,
    {
        self.on_change = Some(Rc::new(RefCell::new(callback)));
        self
    }

    /// Generate CSS for the text field
    pub fn generate_css(&self) -> String {
        r#"
            .textfield {
                padding: 8px 12px;
                border: 1px solid #ced4da;
                border-radius: 4px;
                font-size: 14px;
                font-family: inherit;
                transition: border-color 0.2s ease, box-shadow 0.2s ease;
                width: 100%;
                box-sizing: border-box;
            }
            .textfield:focus {
                outline: none;
                border-color: #007bff;
                box-shadow: 0 0 0 2px rgba(0, 123, 255, 0.25);
            }
            .textfield:disabled {
                background-color: #e9ecef;
                cursor: not-allowed;
            }
        "#.to_string()
    }
}

impl crate::widget::Widget for TextField {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("type".to_string(), "text".to_string());
        attributes.insert("class".to_string(), "textfield".to_string());

        if !self.value.is_empty() {
            attributes.insert("value".to_string(), self.value.clone());
        }

        if !self.placeholder.is_empty() {
            attributes.insert("placeholder".to_string(), self.placeholder.clone());
        }

        if self.disabled {
            attributes.insert("disabled".to_string(), "true".to_string());
        }

        if let Some(max_len) = self.max_length {
            attributes.insert("maxlength".to_string(), max_len.to_string());
        }

        VDomNode {
            node_type: crate::vdom::NodeType::Element,
            tag_name: Some("input".to_string()),
            attributes,
            children: Vec::new(),
            text_content: None,
            event_listeners: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            key: None,
        }
    }
}

/// Checkbox widget for boolean input
#[derive(Clone)]
pub struct Checkbox {
    pub checked: bool,
    pub label: String,
    pub disabled: bool,
    pub on_change: Option<Rc<RefCell<dyn FnMut(bool)>>>,
}

impl std::fmt::Debug for Checkbox {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Checkbox")
            .field("checked", &self.checked)
            .field("label", &self.label)
            .field("disabled", &self.disabled)
            .field("on_change", &self.on_change.is_some())
            .finish()
    }
}

impl Checkbox {
    /// Create a new checkbox
    pub fn new() -> Self {
        Self {
            checked: false,
            label: String::new(),
            disabled: false,
            on_change: None,
        }
    }

    /// Set the checked state
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Set the label text
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set the change handler
    pub fn on_change<F>(mut self, callback: F) -> Self
    where
        F: FnMut(bool) + 'static,
    {
        self.on_change = Some(Rc::new(RefCell::new(callback)));
        self
    }

    /// Generate CSS for the checkbox
    pub fn generate_css(&self) -> String {
        r#"
            .checkbox {
                display: flex;
                align-items: center;
                gap: 8px;
                cursor: pointer;
            }
            .checkbox-input {
                margin: 0;
                cursor: inherit;
            }
            .checkbox-label {
                cursor: inherit;
                user-select: none;
            }
            .checkbox:disabled {
                cursor: not-allowed;
                opacity: 0.6;
            }
        "#.to_string()
    }
}

impl crate::widget::Widget for Checkbox {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        let mut container_attributes = std::collections::HashMap::new();
        container_attributes.insert("class".to_string(), "checkbox".to_string());

        if self.disabled {
            container_attributes.insert("disabled".to_string(), "true".to_string());
        }

        let mut children = Vec::new();

        // Input element
        let mut input_attributes = std::collections::HashMap::new();
        input_attributes.insert("type".to_string(), "checkbox".to_string());
        input_attributes.insert("class".to_string(), "checkbox-input".to_string());

        if self.checked {
            input_attributes.insert("checked".to_string(), "true".to_string());
        }

        if self.disabled {
            input_attributes.insert("disabled".to_string(), "true".to_string());
        }

        children.push(VDomNode {
            node_type: crate::vdom::NodeType::Element,
            tag_name: Some("input".to_string()),
            attributes: input_attributes,
            children: Vec::new(),
            text_content: None,
            event_listeners: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            key: None,
        });

        // Label element (if provided)
        if !self.label.is_empty() {
            let mut label_attributes = std::collections::HashMap::new();
            label_attributes.insert("class".to_string(), "checkbox-label".to_string());

            children.push(VDomNode {
                node_type: crate::vdom::NodeType::Element,
                tag_name: Some("label".to_string()),
                attributes: label_attributes,
                children: Vec::new(),
                text_content: Some(self.label.clone()),
                event_listeners: std::collections::HashMap::new(),
                properties: std::collections::HashMap::new(),
                key: None,
            });
        }

        VDomNode {
            node_type: crate::vdom::NodeType::Element,
            tag_name: Some("div".to_string()),
            attributes: container_attributes,
            children,
            text_content: None,
            event_listeners: std::collections::HashMap::new(),
            properties: std::collections::HashMap::new(),
            key: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::widget::Widget;

    mod button_tests {
        use super::*;

        #[test]
        fn test_button_creation() {
            let button = Button::new("Test Button");
            assert_eq!(button.text, "Test Button");
            assert_eq!(button.variant, ButtonVariant::Primary);
            assert!(!button.disabled);
            assert!(button.on_click.is_none());
        }

        #[test]
        fn test_button_variants() {
            let primary = Button::new("Primary").variant(ButtonVariant::Primary);
            let secondary = Button::new("Secondary").variant(ButtonVariant::Secondary);
            let outline = Button::new("Outline").variant(ButtonVariant::Outline);
            let ghost = Button::new("Ghost").variant(ButtonVariant::Ghost);

            assert_eq!(primary.variant, ButtonVariant::Primary);
            assert_eq!(secondary.variant, ButtonVariant::Secondary);
            assert_eq!(outline.variant, ButtonVariant::Outline);
            assert_eq!(ghost.variant, ButtonVariant::Ghost);
        }

        #[test]
        fn test_button_disabled_state() {
            let disabled_button = Button::new("Disabled").disabled(true);
            let enabled_button = Button::new("Enabled").disabled(false);

            assert!(disabled_button.disabled);
            assert!(!enabled_button.disabled);
        }

        #[test]
        fn test_button_click_handler() {
            let button = Button::new("Click Me").on_click(|| {
                // Just test that the callback is set
            });

            assert!(button.on_click.is_some());
        }

        #[test]
        fn test_button_build() {
            let context = BuildContext::new();
            let button = Button::new("Test").variant(ButtonVariant::Secondary).disabled(true);

            let node = button.build(&context);

            assert_eq!(node.tag_name, Some("button".to_string()));
            assert_eq!(node.text_content, Some("Test".to_string()));
            assert!(node.attributes.contains_key("disabled"));
            assert_eq!(node.attributes.get("class"), Some(&"button button-secondary".to_string()));
        }

        #[test]
        fn test_button_css_generation() {
            let button = Button::new("Test").variant(ButtonVariant::Primary);
            let css = button.generate_css();

            assert!(css.contains(".button"));
            assert!(css.contains(".button-primary"));
            assert!(css.contains("background-color"));
            assert!(css.contains("border"));
            assert!(css.contains("padding"));
        }
    }

    mod textfield_tests {
        use super::*;

        #[test]
        fn test_textfield_creation() {
            let textfield = TextField::new();
            assert_eq!(textfield.value, "");
            assert_eq!(textfield.placeholder, "");
            assert!(!textfield.disabled);
            assert!(textfield.on_change.is_none());
            assert!(textfield.max_length.is_none());
        }

        #[test]
        fn test_textfield_with_value() {
            let textfield = TextField::new()
                .value("Hello World")
                .placeholder("Enter text");

            assert_eq!(textfield.value, "Hello World");
            assert_eq!(textfield.placeholder, "Enter text");
        }

        #[test]
        fn test_textfield_max_length() {
            let textfield = TextField::new().max_length(50);
            assert_eq!(textfield.max_length, Some(50));
        }

        #[test]
        fn test_textfield_disabled_state() {
            let disabled = TextField::new().disabled(true);
            let enabled = TextField::new().disabled(false);

            assert!(disabled.disabled);
            assert!(!enabled.disabled);
        }

        #[test]
        fn test_textfield_change_handler() {
            let textfield = TextField::new().on_change(|_value| {
                // Just test that the callback is set
            });

            assert!(textfield.on_change.is_some());
        }

        #[test]
        fn test_textfield_build() {
            let context = BuildContext::new();
            let textfield = TextField::new()
                .value("test value")
                .placeholder("test placeholder")
                .max_length(100)
                .disabled(true);

            let node = textfield.build(&context);

            assert_eq!(node.tag_name, Some("input".to_string()));
            assert_eq!(node.attributes.get("type"), Some(&"text".to_string()));
            assert_eq!(node.attributes.get("value"), Some(&"test value".to_string()));
            assert_eq!(node.attributes.get("placeholder"), Some(&"test placeholder".to_string()));
            assert_eq!(node.attributes.get("maxlength"), Some(&"100".to_string()));
            assert!(node.attributes.contains_key("disabled"));
        }

        #[test]
        fn test_textfield_css_generation() {
            let textfield = TextField::new();
            let css = textfield.generate_css();

            assert!(css.contains(".textfield"));
            assert!(css.contains("border"));
            assert!(css.contains("padding"));
            assert!(css.contains("font-size"));
        }

        #[test]
        fn test_textfield_validation() {
            // Test max length validation
            let textfield = TextField::new().max_length(5);
            let long_value = "This is too long";

            // In a real implementation, this would be handled by the DOM
            // Here we just test the configuration
            assert_eq!(textfield.max_length, Some(5));
        }
    }

    mod checkbox_tests {
        use super::*;

        #[test]
        fn test_checkbox_creation() {
            let checkbox = Checkbox::new();
            assert!(!checkbox.checked);
            assert_eq!(checkbox.label, "");
            assert!(!checkbox.disabled);
            assert!(checkbox.on_change.is_none());
        }

        #[test]
        fn test_checkbox_with_label() {
            let checkbox = Checkbox::new()
                .label("Accept terms")
                .checked(true);

            assert_eq!(checkbox.label, "Accept terms");
            assert!(checkbox.checked);
        }

        #[test]
        fn test_checkbox_toggle() {
            let unchecked = Checkbox::new().checked(false);
            let checked = Checkbox::new().checked(true);

            assert!(!unchecked.checked);
            assert!(checked.checked);
        }

        #[test]
        fn test_checkbox_disabled_state() {
            let disabled = Checkbox::new().disabled(true);
            let enabled = Checkbox::new().disabled(false);

            assert!(disabled.disabled);
            assert!(!enabled.disabled);
        }

        #[test]
        fn test_checkbox_change_handler() {
            let checkbox = Checkbox::new().on_change(|_checked| {
                // Just test that the callback is set
            });

            assert!(checkbox.on_change.is_some());
        }

        #[test]
        fn test_checkbox_build() {
            let context = BuildContext::new();
            let checkbox = Checkbox::new()
                .label("Test Label")
                .checked(true)
                .disabled(true);

            let node = checkbox.build(&context);

            // Checkbox creates a container with input and label
            assert_eq!(node.tag_name, Some("div".to_string()));
            assert_eq!(node.children.len(), 2); // input and label

            // Check input element
            let input_node = &node.children[0];
            assert_eq!(input_node.tag_name, Some("input".to_string()));
            assert_eq!(input_node.attributes.get("type"), Some(&"checkbox".to_string()));
            assert_eq!(input_node.attributes.get("checked"), Some(&"true".to_string()));
            assert!(input_node.attributes.contains_key("disabled"));

            // Check label element
            let label_node = &node.children[1];
            assert_eq!(label_node.tag_name, Some("label".to_string()));
            assert_eq!(label_node.text_content, Some("Test Label".to_string()));
        }

        #[test]
        fn test_checkbox_css_generation() {
            let checkbox = Checkbox::new();
            let css = checkbox.generate_css();

            assert!(css.contains(".checkbox"));
            assert!(css.contains(".checkbox-input"));
            assert!(css.contains(".checkbox-label"));
            assert!(css.contains("margin"));
            assert!(css.contains("cursor"));
        }

        #[test]
        fn test_checkbox_without_label() {
            let context = BuildContext::new();
            let checkbox = Checkbox::new().checked(true);

            let node = checkbox.build(&context);

            // Should still have input but no label
            assert_eq!(node.children.len(), 1);
            let input_node = &node.children[0];
            assert_eq!(input_node.tag_name, Some("input".to_string()));
            assert_eq!(input_node.attributes.get("type"), Some(&"checkbox".to_string()));
        }
    }

    mod integration_tests {
        use super::*;

        #[test]
        fn test_input_widget_lifecycle() {
            let context = BuildContext::new();

            // Test button lifecycle
            let button = Button::new("Test");
            let button_node = button.build(&context);
            assert!(button_node.tag_name.is_some());

            // Test textfield lifecycle
            let textfield = TextField::new();
            let textfield_node = textfield.build(&context);
            assert!(textfield_node.tag_name.is_some());

            // Test checkbox lifecycle
            let checkbox = Checkbox::new();
            let checkbox_node = checkbox.build(&context);
            assert!(checkbox_node.tag_name.is_some());
        }

        #[test]
        fn test_input_widget_css_consistency() {
            let button_css = Button::new("Test").generate_css();
            let textfield_css = TextField::new().generate_css();
            let checkbox_css = Checkbox::new().generate_css();

            // All should contain basic CSS properties
            assert!(button_css.contains("padding"));
            assert!(textfield_css.contains("border"));
            assert!(checkbox_css.contains("margin"));

            // All should have their respective class names
            assert!(button_css.contains(".button"));
            assert!(textfield_css.contains(".textfield"));
            assert!(checkbox_css.contains(".checkbox"));
        }
    }
}