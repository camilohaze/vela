/**
 * Layout Widgets Example
 *
 * This example demonstrates the usage of Vela's layout widgets:
 * - Container: For padding, margin, and decoration
 * - Row: For horizontal layouts
 * - Column: For vertical layouts
 * - Stack: For positioned overlays
 *
 * Jira: VELA-055
 * Author: GitHub Copilot
 */

use vela_ui::{
    Container, Row, Column, Stack, PositionedChild, Text,
    EdgeInsets, Alignment, MainAxisAlignment, CrossAxisAlignment,
    BoxConstraints, Size
};

/// Example demonstrating Container widget usage
pub fn container_example() -> Container {
    Container::new()
        .child(
            Text::new("Hello Vela!")
        )
        .width(200.0)
        .height(100.0)
        .padding(EdgeInsets::all(16.0))
        .margin(EdgeInsets::symmetric(8.0, 16.0))
        .alignment(Alignment::Center)
}

/// Example demonstrating Row widget for horizontal layout
pub fn row_example() -> Row {
    Row::new()
        .children(vec![
            Box::new(
                Container::new()
                    .child(Text::new("Item 1"))
                    .padding(EdgeInsets::all(8.0))
            ),
            Box::new(
                Container::new()
                    .child(Text::new("Item 2"))
                    .padding(EdgeInsets::all(8.0))
            ),
            Box::new(
                Container::new()
                    .child(Text::new("Item 3"))
                    .padding(EdgeInsets::all(8.0))
            ),
        ])
        .main_axis_alignment(MainAxisAlignment::SpaceEvenly)
        .cross_axis_alignment(CrossAxisAlignment::Center)
}

/// Example demonstrating Column widget for vertical layout
pub fn column_example() -> Column {
    Column::new()
        .children(vec![
            Box::new(
                Container::new()
                    .child(Text::new("Header"))
                    .padding(EdgeInsets::all(16.0))
            ),
            Box::new(
                Container::new()
                    .child(Text::new("Content"))
                    .padding(EdgeInsets::all(16.0))
            ),
            Box::new(
                Container::new()
                    .child(Text::new("Footer"))
                    .padding(EdgeInsets::all(16.0))
            ),
        ])
        .main_axis_alignment(MainAxisAlignment::Start)
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
}

/// Example demonstrating Stack widget for positioned layout
pub fn stack_example() -> Stack {
    Stack::new()
        .children(vec![
            // Background layer
            PositionedChild::new(
                Container::new()
                    .child(Text::new("Background"))
                    .width(300.0)
                    .height(200.0)
                    .padding(EdgeInsets::all(16.0))
            ),

            // Top-left positioned element
            PositionedChild::positioned(
                Container::new()
                    .child(Text::new("Top Left"))
                    .padding(EdgeInsets::all(8.0)),
                Some(20.0), // left
                Some(20.0), // top
                None,       // right
                None        // bottom
            ),

            // Bottom-right positioned element
            PositionedChild::positioned(
                Container::new()
                    .child(Text::new("Bottom Right"))
                    .padding(EdgeInsets::all(8.0)),
                None,       // left
                None,       // top
                Some(20.0), // right
                Some(20.0)  // bottom
            ),

            // Center positioned element
            PositionedChild::positioned(
                Container::new()
                    .child(Text::new("Center"))
                    .padding(EdgeInsets::all(8.0)),
                Some(120.0), // left (centered horizontally)
                Some(80.0),  // top (centered vertically)
                None,        // right
                None         // bottom
            ).width(60.0).height(40.0),
        ])
        .alignment(Alignment::TopLeft)
}

/// Example demonstrating nested layouts
pub fn nested_layout_example() -> Column {
    Column::new()
        .children(vec![
            // Header
            Box::new(
                Container::new()
                    .child(Text::new("App Header"))
                    .padding(EdgeInsets::all(16.0))
            ),

            // Main content with row layout
            Box::new(
                Row::new()
                    .children(vec![
                        // Sidebar
                        Box::new(
                            Container::new()
                                .child(
                                    Column::new()
                                        .children(vec![
                                            Box::new(Text::new("Menu 1")),
                                            Box::new(Text::new("Menu 2")),
                                            Box::new(Text::new("Menu 3")),
                                        ])
                                        .main_axis_alignment(MainAxisAlignment::Start)
                                )
                                .width(150.0)
                                .padding(EdgeInsets::all(8.0))
                        ),

                        // Main content area
                        Box::new(
                            Container::new()
                                .child(
                                    Stack::new()
                                        .children(vec![
                                            PositionedChild::new(
                                                Container::new()
                                                    .child(Text::new("Main Content Area"))
                                                    .padding(EdgeInsets::all(16.0))
                                            ),
                                            PositionedChild::positioned(
                                                Container::new()
                                                    .child(Text::new("Floating Action"))
                                                    .padding(EdgeInsets::all(8.0)),
                                                None, None, Some(16.0), Some(16.0)
                                            ),
                                        ])
                                )
                                .padding(EdgeInsets::all(16.0))
                        ),
                    ])
                    .main_axis_alignment(MainAxisAlignment::Start)
                    .cross_axis_alignment(CrossAxisAlignment::Stretch)
            ),

            // Footer
            Box::new(
                Container::new()
                    .child(Text::new("App Footer"))
                    .padding(EdgeInsets::all(16.0))
            ),
        ])
        .main_axis_alignment(MainAxisAlignment::SpaceBetween)
        .cross_axis_alignment(CrossAxisAlignment::Stretch)
}

/// Example demonstrating layout size calculations
pub fn layout_size_example() {
    let constraints = BoxConstraints::new(0.0, 800.0, 0.0, 600.0);

    // Container with padding and margin
    let container = Container::new()
        .child(Text::new("Test"))
        .width(200.0)
        .height(100.0)
        .padding(EdgeInsets::all(10.0))
        .margin(EdgeInsets::all(5.0));

    let container_size = container.layout_size(&constraints);
    println!("Container size: {}x{}", container_size.width, container_size.height);
    // Expected: 230x130 (200 + 20 padding + 10 margin, 100 + 20 padding + 10 margin)

    // Row with multiple children
    let row = Row::new()
        .children(vec![
            Box::new(Text::new("Item 1")),
            Box::new(Text::new("Item 2")),
            Box::new(Text::new("Item 3")),
        ]);

    let row_size = row.layout_size(&constraints);
    println!("Row size: {}x{}", row_size.width, row_size.height);
    // Expected: 800x20 (max width, max child height)

    // Column with multiple children
    let column = Column::new()
        .children(vec![
            Box::new(Text::new("Item 1")),
            Box::new(Text::new("Item 2")),
        ]);

    let column_size = column.layout_size(&constraints);
    println!("Column size: {}x{}", column_size.width, column_size.height);
    // Expected: 50x600 (max child width, max height)

    // Stack with positioned children
    let stack = Stack::new()
        .children(vec![
            PositionedChild::positioned(
                Text::new("Positioned"),
                Some(100.0), Some(50.0), None, None
            ).width(150.0).height(75.0),
        ]);

    let stack_size = stack.layout_size(&constraints);
    println!("Stack size: {}x{}", stack_size.width, stack_size.height);
    // Expected: 250x125 (100 + 150, 50 + 75)
}

#[cfg(test)]
mod tests {
    use super::*;
    use vela_ui::BuildContext;

    #[test]
    fn test_container_example() {
        let container = container_example();
        let context = BuildContext::new();
        let node = container.build(&context);

        assert_eq!(node.node_type, vela_ui::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert_eq!(node.attributes.get("class"), Some(&"vela-container".to_string()));
    }

    #[test]
    fn test_row_example() {
        let row = row_example();
        let context = BuildContext::new();
        let node = row.build(&context);

        assert_eq!(node.node_type, vela_ui::vdom::NodeType::Element);
        assert_eq!(node.children.len(), 3);
        assert_eq!(node.attributes.get("class"), Some(&"vela-row".to_string()));
    }

    #[test]
    fn test_column_example() {
        let column = column_example();
        let context = BuildContext::new();
        let node = column.build(&context);

        assert_eq!(node.node_type, vela_ui::vdom::NodeType::Element);
        assert_eq!(node.children.len(), 3);
        assert_eq!(node.attributes.get("class"), Some(&"vela-column".to_string()));
    }

    #[test]
    fn test_stack_example() {
        let stack = stack_example();
        let context = BuildContext::new();
        let node = stack.build(&context);

        assert_eq!(node.node_type, vela_ui::vdom::NodeType::Element);
        assert_eq!(node.children.len(), 4);
        assert_eq!(node.attributes.get("class"), Some(&"vela-stack".to_string()));
    }

    #[test]
    fn test_nested_layout_example() {
        let layout = nested_layout_example();
        let context = BuildContext::new();
        let node = layout.build(&context);

        assert_eq!(node.node_type, vela_ui::vdom::NodeType::Element);
        assert_eq!(node.children.len(), 3); // Header, Row, Footer
    }

    #[test]
    fn test_layout_sizes() {
        layout_size_example();
        // Test passes if no panics occur during size calculations
    }
}

fn main() {
    println!("Vela Layout Widgets Examples");
    println!("============================");

    // Run layout size example
    layout_size_example();

    println!("\nExamples completed successfully!");
}