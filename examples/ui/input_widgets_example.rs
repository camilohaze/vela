//! Input Widgets Example
//!
//! This example demonstrates the usage of input widgets in Vela UI:
//! - Button: Clickable buttons with different variants
//! - TextField: Text input fields with validation
//! - Checkbox: Boolean toggle inputs

use vela_ui::layout::{Alignment, EdgeInsets};
use vela_ui::widget::{Button, Checkbox, Column, Container, Row, TextField};

fn main() {
    println!("Vela Input Widgets Example");
    println!("==========================");

    // Example 1: Button variants
    println!("\n1. Button Variants:");
    let primary_button = Button::new("Primary Button")
        .variant(vela_ui::widget::ButtonVariant::Primary);

    let secondary_button = Button::new("Secondary Button")
        .variant(vela_ui::widget::ButtonVariant::Secondary);

    let outline_button = Button::new("Outline Button")
        .variant(vela_ui::widget::ButtonVariant::Outline);

    let ghost_button = Button::new("Ghost Button")
        .variant(vela_ui::widget::ButtonVariant::Ghost);

    let disabled_button = Button::new("Disabled Button")
        .disabled(true);

    println!("Primary Button: {:?}", primary_button.text);
    println!("Secondary Button: {:?}", secondary_button.text);
    println!("Outline Button: {:?}", outline_button.text);
    println!("Ghost Button: {:?}", ghost_button.text);
    println!("Disabled Button: {:?}", disabled_button.disabled);

    // Example 2: Button with click handler
    println!("\n2. Button with Click Handler:");
    let mut click_count = 0;
    let clickable_button = Button::new("Click Me!")
        .on_click(|| {
            click_count += 1;
            println!("Button clicked! Count: {}", click_count);
        });

    // Simulate clicks
    if let Some(callback) = &clickable_button.on_click {
        callback();
        callback();
        callback();
    }

    // Example 3: TextField variations
    println!("\n3. TextField Examples:");
    let basic_textfield = TextField::new()
        .placeholder("Enter your name");

    let prefilled_textfield = TextField::new()
        .value("John Doe")
        .placeholder("Full Name");

    let limited_textfield = TextField::new()
        .placeholder("Username (max 20 chars)")
        .max_length(20);

    let disabled_textfield = TextField::new()
        .value("Cannot edit this")
        .disabled(true);

    println!("Basic TextField placeholder: {:?}", basic_textfield.placeholder);
    println!("Prefilled TextField value: {:?}", prefilled_textfield.value);
    println!("Limited TextField max_length: {:?}", limited_textfield.max_length);
    println!("Disabled TextField: {:?}", disabled_textfield.disabled);

    // Example 4: TextField with change handler
    println!("\n4. TextField with Change Handler:");
    let mut current_value = String::new();
    let reactive_textfield = TextField::new()
        .placeholder("Type something...")
        .on_change(|value| {
            current_value = value.clone();
            println!("Text changed to: {}", value);
        });

    // Simulate text changes
    if let Some(callback) = &reactive_textfield.on_change {
        callback("Hello".to_string());
        callback("Hello World".to_string());
        callback("Hello World!".to_string());
    }

    // Example 5: Checkbox variations
    println!("\n5. Checkbox Examples:");
    let unchecked_checkbox = Checkbox::new();

    let checked_checkbox = Checkbox::new()
        .checked(true);

    let labeled_checkbox = Checkbox::new()
        .checked(false)
        .label("I agree to the terms");

    let disabled_checkbox = Checkbox::new()
        .checked(true)
        .label("Cannot change this")
        .disabled(true);

    println!("Unchecked Checkbox: {:?}", unchecked_checkbox.checked);
    println!("Checked Checkbox: {:?}", checked_checkbox.checked);
    println!("Labeled Checkbox: {:?}", labeled_checkbox.label);
    println!("Disabled Checkbox: {:?}", disabled_checkbox.disabled);

    // Example 6: Checkbox with change handler
    println!("\n6. Checkbox with Change Handler:");
    let mut agreement_accepted = false;
    let agreement_checkbox = Checkbox::new()
        .label("I accept the terms and conditions")
        .on_change(|checked| {
            agreement_accepted = checked;
            println!("Agreement accepted: {}", checked);
        });

    // Simulate checkbox changes
    if let Some(callback) = &agreement_checkbox.on_change {
        callback(true);
        callback(false);
        callback(true);
    }

    // Example 7: Complete form layout
    println!("\n7. Complete Form Layout:");
    let form = Column::new()
        .children(vec![
            // Title
            Container::new()
                .child(vela_ui::widget::TestText::new("User Registration"))
                .padding(EdgeInsets::all(16.0)),

            // Name field
            TextField::new()
                .placeholder("Full Name")
                .max_length(100),

            // Email field
            TextField::new()
                .placeholder("Email Address"),

            // Age field
            TextField::new()
                .placeholder("Age")
                .max_length(3),

            // Terms checkbox
            Checkbox::new()
                .label("I agree to the terms and conditions")
                .checked(false),

            // Newsletter checkbox
            Checkbox::new()
                .label("Subscribe to newsletter")
                .checked(true),

            // Buttons row
            Row::new()
                .children(vec![
                    Button::new("Cancel")
                        .variant(vela_ui::widget::ButtonVariant::Outline),

                    Button::new("Register")
                        .variant(vela_ui::widget::ButtonVariant::Primary)
                        .on_click(|| {
                            println!("Registration submitted!");
                        }),
                ])
                .main_axis_alignment(vela_ui::layout::MainAxisAlignment::SpaceBetween),
        ]);

    println!("Form created with {} children", form.children.len());

    // Example 8: Build and inspect VDOM
    println!("\n8. VDOM Generation:");
    let context = vela_ui::context::BuildContext::new();

    // Build a simple button
    let button_node = primary_button.build(&context);
    println!("Button node type: {:?}", button_node.node_type);
    println!("Button tag: {:?}", button_node.tag_name);
    println!("Button text: {:?}", button_node.text_content);

    // Build a textfield
    let textfield_node = basic_textfield.build(&context);
    println!("TextField node type: {:?}", textfield_node.node_type);
    println!("TextField tag: {:?}", textfield_node.tag_name);
    println!("TextField attributes: {:?}", textfield_node.attributes);

    // Build a checkbox
    let checkbox_node = labeled_checkbox.build(&context);
    println!("Checkbox node type: {:?}", checkbox_node.node_type);
    println!("Checkbox children: {}", checkbox_node.children.len());

    println!("\n✅ All input widget examples completed successfully!");
}</content>
<parameter name="filePath">C:\Users\cristian.naranjo\Downloads\Vela\examples\ui\input_widgets_example.rs