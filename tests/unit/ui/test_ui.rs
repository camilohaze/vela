/*
Tests unitarios básicos para el UI Framework de Vela

Jira: VELA-053
Historia: TASK-053 - Diseñar arquitectura de widgets
Fecha: 2025-01-09
*/

use vela_ui::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_widget() {
        let text = Text::new("Hello World");
        let context = BuildContext::new();

        let node = text.build(&context);
        assert_eq!(node.node_type, crate::vdom::NodeType::Text);
        assert_eq!(node.text_content, Some("Hello World".to_string()));
    }

    #[test]
    fn test_container_widget() {
        let container = Container::new();
        let context = BuildContext::new();

        let node = container.build(&context);
        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
    }

    #[test]
    fn test_vdom_tree() {
        let tree = VDomTree::new(Text::new("Root"));
        assert!(!tree.needs_update());
        assert_eq!(tree.root.node_type, crate::vdom::NodeType::Text);
    }

    #[test]
    fn test_build_context() {
        let context = BuildContext::new();
        assert_eq!(context.depth(), 0);
    }

    #[test]
    fn test_key_creation() {
        let key1 = Key::string("test");
        let key2 = Key::int(42);

        match key1 {
            Key::String(s) => assert_eq!(s, "test"),
            _ => panic!("Expected String key"),
        }

        match key2 {
            Key::Int(i) => assert_eq!(i, 42),
            _ => panic!("Expected Int key"),
        }
    }
}