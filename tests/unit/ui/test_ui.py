"""
Tests unitarios para el UI Framework de Vela

Jira: VELA-053
Historia: TASK-053 - Diseñar arquitectura de widgets
"""

import pytest
from runtime.ui import (
    Widget, StatelessWidget, StatefulWidget, Container, Text,
    VDomNode, VDomTree, BuildContext, Key,
    diff_trees, apply_patches, LifecycleManager, LifecycleState
)


class TestWidgetArchitecture:
    """Suite de tests para la arquitectura de widgets."""

    def test_widget_trait(self):
        """Test que los widgets implementan el trait Widget."""
        text = Text("Hello")
        context = BuildContext()

        node = text.build(context)
        assert isinstance(node, VDomNode)
        assert node.node_type == "Text"
        assert node.text_content == "Hello"

    def test_stateless_widget(self):
        """Test de StatelessWidget base."""
        widget = StatelessWidget()
        context = BuildContext()

        node = widget.build(context)
        assert isinstance(node, VDomNode)
        assert node.node_type == "Empty"

    def test_stateful_widget(self):
        """Test de StatefulWidget base."""
        widget = StatefulWidget()
        context = BuildContext()

        node = widget.build(context)
        assert isinstance(node, VDomNode)
        assert node.node_type == "Empty"

    def test_container_widget(self):
        """Test del widget Container."""
        container = Container().child(Text("Child"))
        context = BuildContext()

        node = container.build(context)
        assert node.node_type == "Element"
        assert node.tag_name == "div"
        assert len(node.children) == 1
        assert node.children[0].text_content == "Child"

    def test_widget_composition(self):
        """Test de composición de widgets."""
        # Crear un árbol de widgets
        root = Container().children([
            Text("Header"),
            Container().children([
                Text("Item 1"),
                Text("Item 2"),
            ]),
            Text("Footer")
        ])

        context = BuildContext()
        node = root.build(context)

        assert node.node_type == "Element"
        assert len(node.children) == 3

        # Header
        assert node.children[0].text_content == "Header"

        # Container with items
        container = node.children[1]
        assert len(container.children) == 2
        assert container.children[0].text_content == "Item 1"
        assert container.children[1].text_content == "Item 2"

        # Footer
        assert node.children[2].text_content == "Footer"


class TestVirtualDOM:
    """Suite de tests para Virtual DOM."""

    def test_vdom_creation(self):
        """Test de creación de nodos VDOM."""
        # Element node
        element = VDomNode.element("div")
        assert element.node_type == "Element"
        assert element.tag_name == "div"

        # Text node
        text = VDomNode.text("Hello")
        assert text.node_type == "Text"
        assert text.text_content == "Hello"

    def test_vdom_attributes(self):
        """Test de atributos en VDOM."""
        node = VDomNode.element("div") \
            .attr("class", "container") \
            .attr("id", "main")

        assert node.attributes["class"] == "container"
        assert node.attributes["id"] == "main"

    def test_vdom_tree(self):
        """Test de VDomTree."""
        tree = VDomTree.new(Text("Root"))
        assert not tree.needs_update()
        assert tree.root.node_type == "Text"
        assert tree.root.text_content == "Root"

    def test_vdom_keys(self):
        """Test de keys en VDOM."""
        key = Key.string("test-key")
        node = VDomNode.element("div").key(key.clone())

        assert node.key == Some(key)

        keys = node.collect_keys()
        assert len(keys) == 1
        assert keys[0] == key


class TestDiffingAlgorithm:
    """Suite de tests para el algoritmo de diffing."""

    def test_identical_nodes_no_patches(self):
        """Test que nodos idénticos no generan patches."""
        node1 = VDomNode.text("Hello")
        node2 = VDomNode.text("Hello")

        patches = diff_nodes(&node1, &node2, &mut vec![], &mut vec![])
        assert patches.is_empty()

    def test_text_content_diff(self):
        """Test de diff de contenido de texto."""
        old_node = VDomNode.text("Old")
        new_node = VDomNode.text("New")

        patches = diff_trees(&VDomTree::new(Text("Old")), &VDomTree::new(Text("New")))

        assert len(patches) == 1
        match &patches[0] {
            Patch::UpdateText { new_text, .. } => assert_eq!(new_text, "New"),
            _ => panic!("Expected UpdateText patch"),
        }

    def test_attribute_diff(self):
        """Test de diff de atributos."""
        old_tree = VDomTree::new(VDomNode.element("div").attr("class", "old"))
        new_tree = VDomTree::new(VDomNode.element("div").attr("class", "new"))

        patches = diff_trees(&old_tree, &new_tree)

        assert len(patches) == 1
        match &patches[0] {
            Patch::UpdateAttributes { attributes, .. } => {
                assert_eq!(attributes["class"], Some("new".to_string()))
            },
            _ => panic!("Expected UpdateAttributes patch"),
        }


class TestLifecycleManagement:
    """Suite de tests para lifecycle management."""

    def test_lifecycle_states(self):
        """Test de estados del lifecycle."""
        manager = LifecycleManager::new()

        # Estado inicial
        assert manager.get_state("test-widget") == LifecycleState::Unmounted

        # Cambiar estado
        manager.set_state("test-widget".to_string(), LifecycleState::Mounted)
        assert manager.get_state("test-widget") == LifecycleState::Mounted

    def test_lifecycle_transitions(self):
        """Test de transiciones del lifecycle."""
        manager = LifecycleManager::new()
        widget = TestLifecycleWidget::new()
        context = BuildContext::new()

        # Mount
        manager.transition("test-widget".to_string(), &mut widget,
                          LifecycleState::Mounting, &context).unwrap()
        assert manager.get_state("test-widget") == LifecycleState::Mounted
        assert widget.mounted

        # Update
        manager.transition("test-widget".to_string(), &mut widget,
                          LifecycleState::Updating, &context).unwrap()
        assert widget.updated

        # Unmount
        manager.transition("test-widget".to_string(), &mut widget,
                          LifecycleState::Unmounting, &context).unwrap()
        assert manager.get_state("test-widget") == LifecycleState::Unmounted
        assert widget.unmounted


class TestBuildContext:
    """Suite de tests para BuildContext."""

    def test_build_context_creation(self):
        """Test de creación de BuildContext."""
        context = BuildContext::new()
        assert context.depth() == 0
        assert context.ancestor_keys.is_empty()

    def test_build_context_inheritance(self):
        """Test de propiedades heredadas en BuildContext."""
        mut context = BuildContext::new()
        context.set_inherited("theme".to_string(), "dark".to_string())

        theme = context.get_inherited("theme")
        assert_eq!(theme, Some("dark".to_string()))

        missing = context.get_inherited("missing")
        assert_eq!(missing, None)


# Widget de prueba para lifecycle tests
#[derive(Debug)]
struct TestLifecycleWidget {
    mounted: bool,
    updated: bool,
    unmounted: bool,
}

impl TestLifecycleWidget {
    fn new() -> Self {
        Self {
            mounted: false,
            updated: false,
            unmounted: false,
        }
    }
}

impl Widget for TestLifecycleWidget {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        VDomNode::text("Test")
    }
}

impl Lifecycle for TestLifecycleWidget {
    fn mount(&mut self, _context: &BuildContext) {
        self.mounted = true
    }

    fn did_update(&mut self, _context: &BuildContext) {
        self.updated = true
    }

    fn will_unmount(&mut self, _context: &BuildContext) {
        self.unmounted = true
    }
}


if __name__ == "__main__":
    pytest.main([__file__, "-v"])