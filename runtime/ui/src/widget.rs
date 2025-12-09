//! Widget trait and base implementations

use crate::vdom::{VDomNode, VDomTree};
use crate::context::BuildContext;
use crate::key::Key;
use crate::lifecycle::{Lifecycle, LifecycleState};
use crate::layout::{BoxConstraints, Size, Offset, EdgeInsets, Alignment};

/// Core trait for all widgets in Vela UI
pub trait Widget: std::fmt::Debug {
    /// Build the widget into a VDOM node
    fn build(&self, context: &BuildContext) -> VDomNode;

    /// Optional key for efficient reconciliation
    fn key(&self) -> Option<Key> {
        None
    }

    /// Type name for debugging
    fn type_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

/// Extension methods for widgets
pub trait WidgetExt: Widget + Sized {
    /// Convert widget to VDOM tree
    fn into_tree(self) -> VDomTree
    where
        Self: 'static,
    {
        VDomTree::new(self)
    }

    /// Build widget with context
    fn build_with_context(self, context: &BuildContext) -> VDomNode
    where
        Self: 'static,
    {
        self.build(context)
    }
}

impl<T: Widget> WidgetExt for T {}

/// Base widget class with lifecycle hooks
#[derive(Debug)]
pub struct BaseWidget {
    pub key: Option<Key>,
    lifecycle_state: LifecycleState,
}

impl BaseWidget {
    /// Create a new base widget
    pub fn new() -> Self {
        Self {
            key: None,
            lifecycle_state: LifecycleState::Unmounted,
        }
    }

    /// Create with key
    pub fn with_key(key: Key) -> Self {
        Self {
            key: Some(key),
            lifecycle_state: LifecycleState::Unmounted,
        }
    }

    /// Get current lifecycle state
    pub fn lifecycle_state(&self) -> LifecycleState {
        self.lifecycle_state.clone()
    }

    /// Protected method for subclasses to override mount behavior
    pub fn on_mount(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Protected method for subclasses to override pre-update behavior
    pub fn on_will_update(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Protected method for subclasses to override post-update behavior
    pub fn on_did_update(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Protected method for subclasses to override pre-unmount behavior
    pub fn on_will_unmount(&mut self, _context: &BuildContext) {
        // Default: no-op
    }

    /// Protected method for subclasses to override update decision
    pub fn should_update(&self, _old_widget: &dyn Widget) -> bool {
        true // Default: always update
    }
}

impl Widget for BaseWidget {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        // Default implementation - subclasses should override
        VDomNode::empty()
    }

    fn key(&self) -> Option<Key> {
        self.key.clone()
    }
}

impl Lifecycle for BaseWidget {
    fn mount(&mut self, context: &BuildContext) {
        self.lifecycle_state = LifecycleState::Mounting;
        self.on_mount(context);
        self.lifecycle_state = LifecycleState::Mounted;
    }

    fn will_update(&mut self, context: &BuildContext) {
        self.lifecycle_state = LifecycleState::Updating;
        self.on_will_update(context);
    }

    fn did_update(&mut self, context: &BuildContext) {
        self.on_did_update(context);
        self.lifecycle_state = LifecycleState::Mounted;
    }

    fn will_unmount(&mut self, context: &BuildContext) {
        self.lifecycle_state = LifecycleState::Unmounting;
        self.on_will_unmount(context);
        // Note: State remains Unmounting until unmount is complete
    }

    fn should_update(&self, old_widget: &dyn Widget) -> bool {
        self.should_update(old_widget)
    }
}

/// Stateless widget base class
#[derive(Debug)]
pub struct StatelessWidget {
    pub key: Option<Key>,
}

impl StatelessWidget {
    pub fn new() -> Self {
        Self { key: None }
    }

    pub fn with_key(key: Key) -> Self {
        Self { key: Some(key) }
    }
}

impl Widget for StatelessWidget {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        VDomNode::empty()
    }

    fn key(&self) -> Option<Key> {
        self.key.clone()
    }
}

/// Stateful widget base class
#[derive(Debug)]
pub struct StatefulWidget {
    pub key: Option<Key>,
    // State would be managed through signals
}

impl StatefulWidget {
    pub fn new() -> Self {
        Self { key: None }
    }

    pub fn with_key(key: Key) -> Self {
        Self { key: Some(key) }
    }
}

impl Widget for StatefulWidget {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        VDomNode::empty()
    }

    fn key(&self) -> Option<Key> {
        self.key.clone()
    }
}

/// Container widget for layout and decoration
#[derive(Debug)]
pub struct Container {
    base: BaseWidget,
    pub child: Option<Box<dyn Widget>>,
    pub width: Option<f32>,
    pub height: Option<f32>,
    pub padding: EdgeInsets,
    pub margin: EdgeInsets,
    pub alignment: Option<Alignment>,
    // TODO: Add decoration support
}

impl Container {
    /// Create a new container
    pub fn new() -> Self {
        Self {
            base: BaseWidget::new(),
            child: None,
            width: None,
            height: None,
            padding: EdgeInsets::all(0.0),
            margin: EdgeInsets::all(0.0),
            alignment: None,
        }
    }

    /// Set the child widget
    pub fn child<W: Widget + 'static>(mut self, child: W) -> Self {
        self.child = Some(Box::new(child));
        self
    }

    /// Set fixed width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set fixed height
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: EdgeInsets) -> Self {
        self.padding = padding;
        self
    }

    /// Set margin
    pub fn margin(mut self, margin: EdgeInsets) -> Self {
        self.margin = margin;
        self
    }

    /// Set alignment for child positioning
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = Some(alignment);
        self
    }

    /// Set key for reconciliation
    pub fn with_key(mut self, key: Key) -> Self {
        self.base = BaseWidget::with_key(key);
        self
    }

    /// Calculate layout size
    pub fn layout_size(&self, constraints: &BoxConstraints) -> Size {
        // Apply margin to constraints
        let content_constraints = BoxConstraints::new(
            (constraints.min_width - self.margin.horizontal_total()).max(0.0),
            (constraints.max_width - self.margin.horizontal_total()).max(0.0),
            (constraints.min_height - self.margin.vertical_total()).max(0.0),
            (constraints.max_height - self.margin.vertical_total()).max(0.0),
        );

        // Apply padding to constraints for child
        let _child_constraints = BoxConstraints::new(
            (content_constraints.min_width - self.padding.horizontal_total()).max(0.0),
            (content_constraints.max_width - self.padding.horizontal_total()).max(0.0),
            (content_constraints.min_height - self.padding.vertical_total()).max(0.0),
            (content_constraints.max_height - self.padding.vertical_total()).max(0.0),
        );

        let child_size = if let Some(_child) = &self.child {
            // For now, assume child has a layout_size method
            // This will be implemented when we add layout traits
            Size::new(50.0, 20.0) // Placeholder
        } else {
            Size::zero()
        };

        // Apply own size constraints
        let width = self.width.unwrap_or(child_size.width + self.padding.horizontal_total());
        let height = self.height.unwrap_or(child_size.height + self.padding.vertical_total());

        // Constrain to provided constraints
        let final_width = width.clamp(content_constraints.min_width, content_constraints.max_width);
        let final_height = height.clamp(content_constraints.min_height, content_constraints.max_height);

        Size::new(final_width, final_height)
    }
}

impl Widget for Container {
    fn build(&self, context: &BuildContext) -> VDomNode {
        let mut node = VDomNode::element("div");

        // Apply CSS classes and styles
        node.attributes.insert("class".to_string(), "vela-container".to_string());

        // Apply size styles if specified
        if let Some(width) = self.width {
            node.attributes.insert("style".to_string(),
                format!("width: {}px;", width));
        }
        if let Some(height) = self.height {
            let current_style = node.attributes.get("style").cloned().unwrap_or_default();
            node.attributes.insert("style".to_string(),
                format!("{} height: {}px;", current_style, height));
        }

        // Apply padding and margin styles
        if self.padding != EdgeInsets::all(0.0) {
            let current_style = node.attributes.get("style").cloned().unwrap_or_default();
            node.attributes.insert("style".to_string(),
                format!("{} padding: {}px {}px {}px {}px;",
                    current_style,
                    self.padding.top, self.padding.right,
                    self.padding.bottom, self.padding.left));
        }

        if self.margin != EdgeInsets::all(0.0) {
            let current_style = node.attributes.get("style").cloned().unwrap_or_default();
            node.attributes.insert("style".to_string(),
                format!("{} margin: {}px {}px {}px {}px;",
                    current_style,
                    self.margin.top, self.margin.right,
                    self.margin.bottom, self.margin.left));
        }

        // Add child if present
        if let Some(child) = &self.child {
            node.children.push(child.build(context));
        }

        node
    }

    fn key(&self) -> Option<Key> {
        self.base.key()
    }
}

impl Lifecycle for Container {
    fn mount(&mut self, context: &BuildContext) {
        self.base.mount(context);
        println!("Container mounted");
    }

    fn will_update(&mut self, context: &BuildContext) {
        self.base.will_update(context);
        println!("Container will update");
    }

    fn did_update(&mut self, context: &BuildContext) {
        self.base.did_update(context);
        println!("Container updated");
    }

    fn will_unmount(&mut self, context: &BuildContext) {
        self.base.will_unmount(context);
        println!("Container will unmount");
    }
}

/// Main axis alignment for flex layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainAxisAlignment {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

/// Cross axis alignment for flex layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CrossAxisAlignment {
    Start,
    Center,
    End,
    Stretch,
    Baseline,
}

/// Main axis size for flex layouts
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainAxisSize {
    Min,
    Max,
}

/// Row widget for horizontal layout
#[derive(Debug)]
pub struct Row {
    base: BaseWidget,
    pub children: Vec<Box<dyn Widget>>,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub main_axis_size: MainAxisSize,
}

impl Row {
    /// Create a new row
    pub fn new() -> Self {
        Self {
            base: BaseWidget::new(),
            children: Vec::new(),
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Center,
            main_axis_size: MainAxisSize::Max,
        }
    }

    /// Add children to the row
    pub fn children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    /// Set main axis alignment (horizontal)
    pub fn main_axis_alignment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }

    /// Set cross axis alignment (vertical)
    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    /// Set main axis size
    pub fn main_axis_size(mut self, size: MainAxisSize) -> Self {
        self.main_axis_size = size;
        self
    }

    /// Set key for reconciliation
    pub fn with_key(mut self, key: Key) -> Self {
        self.base = BaseWidget::with_key(key);
        self
    }

    /// Calculate layout size for row
    pub fn layout_size(&self, constraints: &BoxConstraints) -> Size {
        if self.children.is_empty() {
            return Size::new(
                if self.main_axis_size == MainAxisSize::Max { constraints.max_width } else { 0.0 },
                0.0
            );
        }

        let mut total_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;

        // Calculate size for each child
        for _child in &self.children {
            // Placeholder: assume child has layout_size method
            let child_size = Size::new(50.0, 20.0); // Placeholder
            total_width += child_size.width;
            max_height = max_height.max(child_size.height);
        }

        // Apply main axis size constraint
        let width = match self.main_axis_size {
            MainAxisSize::Max => constraints.max_width,
            MainAxisSize::Min => total_width,
        }.clamp(constraints.min_width, constraints.max_width);

        let height = max_height.clamp(constraints.min_height, constraints.max_height);

        Size::new(width, height)
    }
}

impl Widget for Row {
    fn build(&self, context: &BuildContext) -> VDomNode {
        let mut node = VDomNode::element("div");

        // Apply CSS classes and styles for flexbox
        node.attributes.insert("class".to_string(), "vela-row".to_string());
        node.attributes.insert("style".to_string(),
            "display: flex; flex-direction: row;".to_string());

        // Apply main axis alignment
        let justify_content = match self.main_axis_alignment {
            MainAxisAlignment::Start => "flex-start",
            MainAxisAlignment::Center => "center",
            MainAxisAlignment::End => "flex-end",
            MainAxisAlignment::SpaceBetween => "space-between",
            MainAxisAlignment::SpaceAround => "space-around",
            MainAxisAlignment::SpaceEvenly => "space-evenly",
        };

        // Apply cross axis alignment
        let align_items = match self.cross_axis_alignment {
            CrossAxisAlignment::Start => "flex-start",
            CrossAxisAlignment::Center => "center",
            CrossAxisAlignment::End => "flex-end",
            CrossAxisAlignment::Stretch => "stretch",
            CrossAxisAlignment::Baseline => "baseline",
        };

        let current_style = node.attributes.get("style").cloned().unwrap_or_default();
        node.attributes.insert("style".to_string(),
            format!("{} justify-content: {}; align-items: {};",
                current_style, justify_content, align_items));

        // Add children
        for child in &self.children {
            node.children.push(child.build(context));
        }

        node
    }

    fn key(&self) -> Option<Key> {
        self.base.key()
    }
}

impl Lifecycle for Row {
    fn mount(&mut self, context: &BuildContext) {
        self.base.mount(context);
        println!("Row mounted");
    }

    fn will_update(&mut self, context: &BuildContext) {
        self.base.will_update(context);
        println!("Row will update");
    }

    fn did_update(&mut self, context: &BuildContext) {
        self.base.did_update(context);
        println!("Row updated");
    }

    fn will_unmount(&mut self, context: &BuildContext) {
        self.base.will_unmount(context);
        println!("Row will unmount");
    }
}

/// Column widget for vertical layout
#[derive(Debug)]
pub struct Column {
    base: BaseWidget,
    pub children: Vec<Box<dyn Widget>>,
    pub main_axis_alignment: MainAxisAlignment,
    pub cross_axis_alignment: CrossAxisAlignment,
    pub main_axis_size: MainAxisSize,
}

impl Column {
    /// Create a new column
    pub fn new() -> Self {
        Self {
            base: BaseWidget::new(),
            children: Vec::new(),
            main_axis_alignment: MainAxisAlignment::Start,
            cross_axis_alignment: CrossAxisAlignment::Center,
            main_axis_size: MainAxisSize::Max,
        }
    }

    /// Add children to the column
    pub fn children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }

    /// Set main axis alignment (vertical)
    pub fn main_axis_alignment(mut self, alignment: MainAxisAlignment) -> Self {
        self.main_axis_alignment = alignment;
        self
    }

    /// Set cross axis alignment (horizontal)
    pub fn cross_axis_alignment(mut self, alignment: CrossAxisAlignment) -> Self {
        self.cross_axis_alignment = alignment;
        self
    }

    /// Set main axis size
    pub fn main_axis_size(mut self, size: MainAxisSize) -> Self {
        self.main_axis_size = size;
        self
    }

    /// Set key for reconciliation
    pub fn with_key(mut self, key: Key) -> Self {
        self.base = BaseWidget::with_key(key);
        self
    }

    /// Calculate layout size for column
    pub fn layout_size(&self, constraints: &BoxConstraints) -> Size {
        if self.children.is_empty() {
            return Size::new(
                0.0,
                if self.main_axis_size == MainAxisSize::Max { constraints.max_height } else { 0.0 }
            );
        }

        let mut max_width: f32 = 0.0;
        let mut total_height: f32 = 0.0;

        // Calculate size for each child
        for _child in &self.children {
            // Placeholder: assume child has layout_size method
            let child_size = Size::new(50.0, 20.0); // Placeholder
            max_width = max_width.max(child_size.width);
            total_height += child_size.height;
        }

        // Apply main axis size constraint
        let height = match self.main_axis_size {
            MainAxisSize::Max => constraints.max_height,
            MainAxisSize::Min => total_height,
        }.clamp(constraints.min_height, constraints.max_height);

        let width = max_width.clamp(constraints.min_width, constraints.max_width);

        Size::new(width, height)
    }
}

impl Widget for Column {
    fn build(&self, context: &BuildContext) -> VDomNode {
        let mut node = VDomNode::element("div");

        // Apply CSS classes and styles for flexbox
        node.attributes.insert("class".to_string(), "vela-column".to_string());
        node.attributes.insert("style".to_string(),
            "display: flex; flex-direction: column;".to_string());

        // Apply main axis alignment (vertical)
        let justify_content = match self.main_axis_alignment {
            MainAxisAlignment::Start => "flex-start",
            MainAxisAlignment::Center => "center",
            MainAxisAlignment::End => "flex-end",
            MainAxisAlignment::SpaceBetween => "space-between",
            MainAxisAlignment::SpaceAround => "space-around",
            MainAxisAlignment::SpaceEvenly => "space-evenly",
        };

        // Apply cross axis alignment (horizontal)
        let align_items = match self.cross_axis_alignment {
            CrossAxisAlignment::Start => "flex-start",
            CrossAxisAlignment::Center => "center",
            CrossAxisAlignment::End => "flex-end",
            CrossAxisAlignment::Stretch => "stretch",
            CrossAxisAlignment::Baseline => "baseline",
        };

        let current_style = node.attributes.get("style").cloned().unwrap_or_default();
        node.attributes.insert("style".to_string(),
            format!("{} justify-content: {}; align-items: {};",
                current_style, justify_content, align_items));

        // Add children
        for child in &self.children {
            node.children.push(child.build(context));
        }

        node
    }

    fn key(&self) -> Option<Key> {
        self.base.key()
    }
}

impl Lifecycle for Column {
    fn mount(&mut self, context: &BuildContext) {
        self.base.mount(context);
        println!("Column mounted");
    }

    fn will_update(&mut self, context: &BuildContext) {
        self.base.will_update(context);
        println!("Column will update");
    }

    fn did_update(&mut self, context: &BuildContext) {
        self.base.did_update(context);
        println!("Column updated");
    }

    fn will_unmount(&mut self, context: &BuildContext) {
        self.base.will_unmount(context);
        println!("Column will unmount");
    }
}

/// Stack widget for positioned layout
#[derive(Debug)]
pub struct Stack {
    base: BaseWidget,
    pub children: Vec<PositionedChild>,
    pub alignment: Alignment,
    pub fit: StackFit,
}

#[derive(Debug)]
pub struct PositionedChild {
    pub child: Box<dyn Widget>,
    pub position: Option<Offset>,
    pub left: Option<f32>,
    pub top: Option<f32>,
    pub right: Option<f32>,
    pub bottom: Option<f32>,
    pub width: Option<f32>,
    pub height: Option<f32>,
}

impl PositionedChild {
    /// Create a non-positioned child
    pub fn new<W: Widget + 'static>(child: W) -> Self {
        Self {
            child: Box::new(child),
            position: None,
            left: None,
            top: None,
            right: None,
            bottom: None,
            width: None,
            height: None,
        }
    }

    /// Create a positioned child with absolute position
    pub fn positioned<W: Widget + 'static>(child: W, left: Option<f32>, top: Option<f32>, right: Option<f32>, bottom: Option<f32>) -> Self {
        Self {
            child: Box::new(child),
            position: Some(Offset::new(left.unwrap_or(0.0), top.unwrap_or(0.0))),
            left,
            top,
            right,
            bottom,
            width: None,
            height: None,
        }
    }

    /// Set width
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    /// Set height
    pub fn height(mut self, height: f32) -> Self {
        self.height = Some(height);
        self
    }
}

/// How the stack should size itself
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StackFit {
    Loose,
    Expand,
    Passthrough,
}

impl Stack {
    /// Create a new stack
    pub fn new() -> Self {
        Self {
            base: BaseWidget::new(),
            children: Vec::new(),
            alignment: Alignment::top_left(),
            fit: StackFit::Loose,
        }
    }

    /// Add children to the stack
    pub fn children(mut self, children: Vec<PositionedChild>) -> Self {
        self.children = children;
        self
    }

    /// Set alignment for non-positioned children
    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Set stack fit
    pub fn fit(mut self, fit: StackFit) -> Self {
        self.fit = fit;
        self
    }

    /// Set key for reconciliation
    pub fn with_key(mut self, key: Key) -> Self {
        self.base = BaseWidget::with_key(key);
        self
    }

    /// Calculate layout size for stack
    pub fn layout_size(&self, constraints: &BoxConstraints) -> Size {
        if self.children.is_empty() {
            return Size::zero();
        }

        let mut max_width: f32 = 0.0;
        let mut max_height: f32 = 0.0;

        // Calculate size based on all children
        for child in &self.children {
            // Placeholder: assume child has layout_size method
            let child_size = Size::new(
                child.width.unwrap_or(50.0),
                child.height.unwrap_or(20.0)
            );

            // Consider positioned children
            if let Some(position) = child.position {
                max_width = max_width.max(position.x + child_size.width);
                max_height = max_height.max(position.y + child_size.height);
            } else {
                max_width = max_width.max(child_size.width);
                max_height = max_height.max(child_size.height);
            }
        }

        // Apply stack fit
        let size = match self.fit {
            StackFit::Loose => Size::new(max_width, max_height),
            StackFit::Expand => Size::new(constraints.max_width, constraints.max_height),
            StackFit::Passthrough => Size::new(constraints.max_width, constraints.max_height),
        };

        Size::new(
            size.width.clamp(constraints.min_width, constraints.max_width),
            size.height.clamp(constraints.min_height, constraints.max_height)
        )
    }
}

impl Widget for Stack {
    fn build(&self, context: &BuildContext) -> VDomNode {
        let mut node = VDomNode::element("div");

        // Apply CSS classes and styles
        node.attributes.insert("class".to_string(), "vela-stack".to_string());
        node.attributes.insert("style".to_string(),
            "position: relative;".to_string());

        // Add children
        for child in &self.children {
            let mut child_node = child.child.build(context);

            // Apply positioning styles if positioned
            if child.position.is_some() || child.left.is_some() || child.top.is_some() ||
               child.right.is_some() || child.bottom.is_some() {

                let mut style = "position: absolute;".to_string();

                if let Some(left) = child.left {
                    style.push_str(&format!(" left: {}px;", left));
                }
                if let Some(top) = child.top {
                    style.push_str(&format!(" top: {}px;", top));
                }
                if let Some(right) = child.right {
                    style.push_str(&format!(" right: {}px;", right));
                }
                if let Some(bottom) = child.bottom {
                    style.push_str(&format!(" bottom: {}px;", bottom));
                }
                if let Some(width) = child.width {
                    style.push_str(&format!(" width: {}px;", width));
                }
                if let Some(height) = child.height {
                    style.push_str(&format!(" height: {}px;", height));
                }

                child_node.attributes.insert("style".to_string(), style);
            } else {
                // Non-positioned children get alignment
                let style = if self.alignment == Alignment::top_left() {
                    "position: absolute; top: 0; left: 0;"
                } else if self.alignment == Alignment::top_center() {
                    "position: absolute; top: 0; left: 50%; transform: translateX(-50%);"
                } else if self.alignment == Alignment::top_right() {
                    "position: absolute; top: 0; right: 0;"
                } else if self.alignment == Alignment::center_left() {
                    "position: absolute; top: 50%; left: 0; transform: translateY(-50%);"
                } else if self.alignment == Alignment::center() {
                    "position: absolute; top: 50%; left: 50%; transform: translate(-50%, -50%);"
                } else if self.alignment == Alignment::center_right() {
                    "position: absolute; top: 50%; right: 0; transform: translateY(-50%);"
                } else if self.alignment == Alignment::bottom_left() {
                    "position: absolute; bottom: 0; left: 0;"
                } else if self.alignment == Alignment::bottom_center() {
                    "position: absolute; bottom: 0; left: 50%; transform: translateX(-50%);"
                } else if self.alignment == Alignment::bottom_right() {
                    "position: absolute; bottom: 0; right: 0;"
                } else {
                    "position: absolute; top: 0; left: 0;" // default fallback
                };
                child_node.attributes.insert("style".to_string(), style.to_string());
            }

            node.children.push(child_node);
        }

        node
    }

    fn key(&self) -> Option<Key> {
        self.base.key()
    }
}

impl Lifecycle for Stack {
    fn mount(&mut self, context: &BuildContext) {
        self.base.mount(context);
        println!("Stack mounted");
    }

    fn will_update(&mut self, context: &BuildContext) {
        self.base.will_update(context);
        println!("Stack will update");
    }

    fn did_update(&mut self, context: &BuildContext) {
        self.base.did_update(context);
        println!("Stack updated");
    }

    fn will_unmount(&mut self, context: &BuildContext) {
        self.base.will_unmount(context);
        println!("Stack will unmount");
    }
}

/// Simple text widget for testing purposes
#[derive(Debug)]
pub struct TestText {
    content: String,
    key: Option<Key>,
}

impl TestText {
    pub fn new<S: Into<String>>(content: S) -> Self {
        Self {
            content: content.into(),
            key: None,
        }
    }

    pub fn with_key(mut self, key: Key) -> Self {
        self.key = Some(key);
        self
    }
}

impl Widget for TestText {
    fn build(&self, _context: &BuildContext) -> VDomNode {
        VDomNode::text(&self.content)
    }

    fn key(&self) -> Option<Key> {
        self.key.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_widget() {
        let text = TestText::new("Hello Vela");
        let context = BuildContext::new();
        let node = text.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Text);
        assert_eq!(node.text_content, Some("Hello Vela".to_string()));
    }

    #[test]
    fn test_container_widget() {
        let container = Container::new()
            .child(TestText::new("Hello World"));

        let context = BuildContext::new();
        let node = container.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.attributes.get("class"), Some(&"vela-container".to_string()));
    }

    #[test]
    fn test_container_with_layout_properties() {
        let container = Container::new()
            .child(TestText::new("Test"))
            .width(200.0)
            .height(100.0)
            .padding(EdgeInsets::all(10.0))
            .margin(EdgeInsets::all(5.0))
            .alignment(Alignment::center());

        let context = BuildContext::new();
        let node = container.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));

        // Check that styles are applied
        let style = node.attributes.get("style").unwrap();
        assert!(style.contains("width: 200px"));
        assert!(style.contains("height: 100px"));
        assert!(style.contains("padding: 10px 10px 10px 10px"));
        assert!(style.contains("margin: 5px 5px 5px 5px"));
    }

    #[test]
    fn test_container_layout_size() {
        let container = Container::new()
            .child(TestText::new("Test"))
            .width(150.0)
            .height(80.0)
            .padding(EdgeInsets::all(10.0))
            .margin(EdgeInsets::all(5.0));

        let constraints = BoxConstraints::new(0.0, 400.0, 0.0, 300.0);
        let size = container.layout_size(&constraints);

        // Size should be the specified width/height (including padding, excluding margin)
        assert_eq!(size.width, 150.0); // Specified width
        assert_eq!(size.height, 80.0); // Specified height
    }

    #[test]
    fn test_container_empty() {
        let container = Container::new();

        let context = BuildContext::new();
        let node = container.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert_eq!(node.children.len(), 0);
    }

    #[test]
    fn test_widget_keys() {
        let key = Key::String("test-key".to_string());
        let text = TestText::new("Test").with_key(key.clone());

        assert_eq!(text.key(), Some(key));
    }

    #[test]
    fn test_base_widget_creation() {
        let widget = BaseWidget::new();
        assert_eq!(widget.lifecycle_state(), LifecycleState::Unmounted);
        assert!(widget.key().is_none());
    }

    #[test]
    fn test_base_widget_with_key() {
        let key = Key::String("test-key".to_string());
        let widget = BaseWidget::with_key(key.clone());
        assert_eq!(widget.key(), Some(key));
        assert_eq!(widget.lifecycle_state(), LifecycleState::Unmounted);
    }

    #[test]
    fn test_base_widget_build_default() {
        let widget = BaseWidget::new();
        let context = BuildContext::new();
        let node = widget.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Empty);
    }

    #[derive(Debug)]
    struct TestLifecycleWidget {
        base: BaseWidget,
        mounted: bool,
        updated: bool,
        unmounted: bool,
    }

    impl TestLifecycleWidget {
        fn new() -> Self {
            Self {
                base: BaseWidget::new(),
                mounted: false,
                updated: false,
                unmounted: false,
            }
        }
    }

    impl Widget for TestLifecycleWidget {
        fn build(&self, _context: &BuildContext) -> VDomNode {
            VDomNode::text("Test Widget")
        }

        fn key(&self) -> Option<Key> {
            self.base.key()
        }
    }

    impl Lifecycle for TestLifecycleWidget {
        fn mount(&mut self, context: &BuildContext) {
            self.base.mount(context);
            self.mounted = true;
        }

        fn will_update(&mut self, context: &BuildContext) {
            self.base.will_update(context);
        }

        fn did_update(&mut self, context: &BuildContext) {
            self.base.did_update(context);
            self.updated = true;
        }

        fn will_unmount(&mut self, context: &BuildContext) {
            self.base.will_unmount(context);
            self.unmounted = true;
        }
    }

    #[test]
    fn test_base_widget_lifecycle_hooks() {
        let mut widget = TestLifecycleWidget::new();
        let context = BuildContext::new();

        // Test mount
        widget.mount(&context);
        assert!(widget.mounted);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Mounted);

        // Test will_update
        widget.will_update(&context);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Updating);

        // Test did_update
        widget.did_update(&context);
        assert!(widget.updated);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Mounted);

        // Test will_unmount
        widget.will_unmount(&context);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Unmounting);
        assert!(widget.unmounted);
    }

    #[test]
    fn test_base_widget_should_update() {
        let widget = BaseWidget::new();
        let old_widget = BaseWidget::new();

        // Default implementation always returns true
        assert!(widget.should_update(&old_widget));
    }

    #[test]
    fn test_base_widget_lifecycle_manager_integration() {
        use crate::lifecycle::LifecycleManager;

        let mut manager = LifecycleManager::new();
        let mut widget = TestLifecycleWidget::new();
        let context = BuildContext::new();

        // Test mounting through lifecycle manager
        manager.transition(
            "test-widget".to_string(),
            &mut widget,
            LifecycleState::Mounting,
            &context
        ).unwrap();

        assert!(widget.mounted);
        assert_eq!(manager.get_state("test-widget"), LifecycleState::Mounted);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Mounted);

        // Test updating through lifecycle manager
        manager.transition(
            "test-widget".to_string(),
            &mut widget,
            LifecycleState::Updating,
            &context
        ).unwrap();

        assert!(widget.updated);
        assert_eq!(manager.get_state("test-widget"), LifecycleState::Mounted);
        assert_eq!(widget.base.lifecycle_state(), LifecycleState::Mounted);
    }

    #[test]
    fn test_row_widget() {
        let row = Row::new()
            .children(vec![
                Box::new(TestText::new("Item 1")),
                Box::new(TestText::new("Item 2")),
                Box::new(TestText::new("Item 3")),
            ])
            .main_axis_alignment(MainAxisAlignment::SpaceBetween)
            .cross_axis_alignment(CrossAxisAlignment::Center);

        let context = BuildContext::new();
        let node = row.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert_eq!(node.children.len(), 3);
        assert_eq!(node.attributes.get("class"), Some(&"vela-row".to_string()));

        let style = node.attributes.get("style").unwrap();
        assert!(style.contains("display: flex"));
        assert!(style.contains("flex-direction: row"));
        assert!(style.contains("justify-content: space-between"));
        assert!(style.contains("align-items: center"));
    }

    #[test]
    fn test_column_widget() {
        let column = Column::new()
            .children(vec![
                Box::new(TestText::new("Item 1")),
                Box::new(TestText::new("Item 2")),
            ])
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Stretch);

        let context = BuildContext::new();
        let node = column.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.attributes.get("class"), Some(&"vela-column".to_string()));

        let style = node.attributes.get("style").unwrap();
        assert!(style.contains("display: flex"));
        assert!(style.contains("flex-direction: column"));
        assert!(style.contains("justify-content: center"));
        assert!(style.contains("align-items: stretch"));
    }

    #[test]
    fn test_stack_widget() {
        let stack = Stack::new()
            .children(vec![
                PositionedChild::new(TestText::new("Background")),
                PositionedChild::positioned(TestText::new("Top Left"), Some(10.0), Some(20.0), None, None),
                PositionedChild::positioned(TestText::new("Bottom Right"), None, None, Some(10.0), Some(20.0))
                    .width(100.0)
                    .height(50.0),
            ])
            .alignment(Alignment::center());

        let context = BuildContext::new();
        let node = stack.build(&context);

        assert_eq!(node.node_type, crate::vdom::NodeType::Element);
        assert_eq!(node.tag_name, Some("div".to_string()));
        assert_eq!(node.children.len(), 3);
        assert_eq!(node.attributes.get("class"), Some(&"vela-stack".to_string()));

        let style = node.attributes.get("style").unwrap();
        assert!(style.contains("position: relative"));

        // Check positioned children styles
        assert!(node.children[1].attributes.get("style").unwrap().contains("position: absolute"));
        assert!(node.children[1].attributes.get("style").unwrap().contains("left: 10px"));
        assert!(node.children[1].attributes.get("style").unwrap().contains("top: 20px"));

        assert!(node.children[2].attributes.get("style").unwrap().contains("position: absolute"));
        assert!(node.children[2].attributes.get("style").unwrap().contains("right: 10px"));
        assert!(node.children[2].attributes.get("style").unwrap().contains("bottom: 20px"));
        assert!(node.children[2].attributes.get("style").unwrap().contains("width: 100px"));
        assert!(node.children[2].attributes.get("style").unwrap().contains("height: 50px"));
    }

    #[test]
    fn test_positioned_child_constructors() {
        let non_positioned = PositionedChild::new(TestText::new("Test"));
        assert!(non_positioned.position.is_none());
        assert!(non_positioned.left.is_none());

        let positioned = PositionedChild::positioned(TestText::new("Test"), Some(10.0), Some(20.0), None, None);
        assert_eq!(positioned.left, Some(10.0));
        assert_eq!(positioned.top, Some(20.0));
        assert!(positioned.right.is_none());
        assert!(positioned.bottom.is_none());

        let sized = positioned.width(100.0).height(50.0);
        assert_eq!(sized.width, Some(100.0));
        assert_eq!(sized.height, Some(50.0));
    }

    #[test]
    fn test_layout_sizes() {
        let constraints = BoxConstraints::new(0.0, 400.0, 0.0, 300.0);

        // Test container layout
        let container = Container::new()
            .child(TestText::new("Test"))
            .width(150.0)
            .height(80.0)
            .padding(EdgeInsets::all(10.0))
            .margin(EdgeInsets::all(5.0));
        let container_size = container.layout_size(&constraints);
        assert_eq!(container_size.width, 150.0); // Specified width
        assert_eq!(container_size.height, 80.0); // Specified height

        // Test row layout
        let row = Row::new()
            .children(vec![
                Box::new(TestText::new("Item 1")),
                Box::new(TestText::new("Item 2")),
            ]);
        let row_size = row.layout_size(&constraints);
        assert_eq!(row_size.width, 400.0); // Max width due to MainAxisSize::Max
        assert_eq!(row_size.height, 20.0); // Max height of children

        // Test column layout
        let column = Column::new()
            .children(vec![
                Box::new(TestText::new("Item 1")),
                Box::new(TestText::new("Item 2")),
            ]);
        let column_size = column.layout_size(&constraints);
        assert_eq!(column_size.width, 50.0); // Max width of children
        assert_eq!(column_size.height, 300.0); // Max height due to MainAxisSize::Max

        // Test stack layout
        let stack = Stack::new()
            .children(vec![
                PositionedChild::positioned(TestText::new("Test"), Some(50.0), Some(30.0), None, None)
                    .width(100.0)
                    .height(60.0),
            ]);
        let stack_size = stack.layout_size(&constraints);
        assert_eq!(stack_size.width, 150.0); // 50 + 100
        assert_eq!(stack_size.height, 90.0); // 30 + 60
    }
}