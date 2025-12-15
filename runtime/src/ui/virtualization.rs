//! Virtualized ListView implementation for efficient rendering of large lists
//!
//! This module provides virtualized list views that can handle thousands of items
//! without performance degradation by only rendering visible items.

use std::collections::HashMap;
use std::ops::Range;

/// Configuration for virtualized list behavior
#[derive(Debug, Clone)]
pub struct VirtualizationConfig {
    /// Height of each item in the list
    pub item_height: f32,
    /// Number of extra items to render outside viewport for smooth scrolling
    pub overscan_count: usize,
    /// Estimated total height for scrollbar calculations
    pub estimated_total_height: Option<f32>,
}

impl Default for VirtualizationConfig {
    fn default() -> Self {
        Self {
            item_height: 50.0,
            overscan_count: 5,
            estimated_total_height: None,
        }
    }
}

/// Represents the visible range of items in a virtualized list
#[derive(Debug, Clone, PartialEq)]
pub struct VisibleRange {
    pub start: usize,
    pub end: usize,
}

impl VisibleRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Manages the viewport and calculates which items should be visible
pub struct ViewportManager {
    config: VirtualizationConfig,
    scroll_top: f32,
    viewport_height: f32,
    total_items: usize,
}

impl ViewportManager {
    pub fn new(config: &VirtualizationConfig, viewport_height: f32, total_items: usize) -> Self {
        Self {
            config: config.clone(),
            scroll_top: 0.0,
            viewport_height,
            total_items,
        }
    }

    pub fn set_scroll_top(&mut self, scroll_top: f32) {
        self.scroll_top = scroll_top.max(0.0);
    }

    pub fn set_viewport_height(&mut self, height: f32) {
        self.viewport_height = height;
    }

    pub fn set_total_items(&mut self, count: usize) {
        self.total_items = count;
    }

    /// Calculate which items should be visible based on current scroll position
    pub fn get_visible_range(&self) -> VisibleRange {
        if self.total_items == 0 {
            return VisibleRange::new(0, 0);
        }

        let start_index = (self.scroll_top / self.config.item_height) as usize;
        let visible_count = (self.viewport_height / self.config.item_height).ceil() as usize;

        let start = start_index.saturating_sub(self.config.overscan_count);
        let end = (start_index + visible_count + self.config.overscan_count).min(self.total_items);

        VisibleRange::new(start, end)
    }

    /// Get the total height of all items
    pub fn get_total_height(&self) -> f32 {
        self.total_items as f32 * self.config.item_height
    }

    /// Get the offset for a specific item
    pub fn get_item_offset(&self, index: usize) -> f32 {
        index as f32 * self.config.item_height
    }

    /// Check if an item is currently visible
    pub fn is_item_visible(&self, index: usize) -> bool {
        let range = self.get_visible_range();
        index >= range.start && index < range.end
    }
}

/// Pool for recycling widget instances to improve performance
pub struct WidgetPool<T> {
    pub available: Vec<T>,
    create_fn: Box<dyn Fn() -> T>,
}

impl<T> WidgetPool<T> {
    pub fn new<F>(create_fn: F) -> Self
    where
        F: Fn() -> T + 'static,
    {
        Self {
            available: Vec::new(),
            create_fn: Box::new(create_fn),
        }
    }

    pub fn get(&mut self) -> T {
        self.available.pop().unwrap_or_else(|| (self.create_fn)())
    }

    pub fn recycle(&mut self, item: T) {
        self.available.push(item);
    }

    pub fn clear(&mut self) {
        self.available.clear();
    }
}

/// Virtualized list view that efficiently renders large lists
pub struct VirtualizedListView<T> {
    config: VirtualizationConfig,
    viewport_manager: ViewportManager,
    items: Vec<T>,
    widget_pool: WidgetPool<Box<dyn Widget>>,
    pub rendered_items: HashMap<usize, Box<dyn Widget>>,
}

pub trait Widget {
    fn render(&self, x: f32, y: f32, width: f32, height: f32);
    fn get_height(&self) -> f32;
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T: 'static> VirtualizedListView<T> {
    pub fn new<F>(config: VirtualizationConfig, items: &[T], create_widget_fn: F) -> Self
    where
        F: Fn(&T) -> Box<dyn Widget> + 'static,
        T: Clone,
    {
        let total_items = items.len();
        let viewport_manager = ViewportManager::new(
            &config,
            600.0, // Default viewport height
            total_items,
        );

        let first_item = items.first().cloned().unwrap_or_else(|| panic!("Items cannot be empty"));
        Self {
            config,
            viewport_manager,
            items: items.to_vec(),
            widget_pool: WidgetPool::new(move || create_widget_fn(&first_item)),
            rendered_items: HashMap::new(),
        }
    }

    pub fn set_viewport_height(&mut self, height: f32) {
        self.viewport_manager.set_viewport_height(height);
    }

    pub fn set_scroll_top(&mut self, scroll_top: f32) {
        self.viewport_manager.set_scroll_top(scroll_top);
        self.update_visible_items();
    }

    pub fn get_total_height(&self) -> f32 {
        self.viewport_manager.get_total_height()
    }

    fn update_visible_items(&mut self) {
        let visible_range = self.viewport_manager.get_visible_range();

        // Remove items that are no longer visible
        let mut to_remove = Vec::new();
        for &index in self.rendered_items.keys() {
            if index < visible_range.start || index >= visible_range.end {
                to_remove.push(index);
            }
        }

        for index in to_remove {
            if let Some(widget) = self.rendered_items.remove(&index) {
                self.widget_pool.recycle(widget);
            }
        }

        // Add new visible items
        for index in visible_range.start..visible_range.end {
            if !self.rendered_items.contains_key(&index) {
                let widget = self.widget_pool.get();
                self.rendered_items.insert(index, widget);
            }
        }
    }

    pub fn render(&self, context: &RenderContext) {
        for (index, widget) in &self.rendered_items {
            let y = self.viewport_manager.get_item_offset(*index) - self.viewport_manager.scroll_top;
            widget.render(0.0, y, context.width, self.config.item_height);
        }
    }

    pub fn get_visible_item_count(&self) -> usize {
        self.rendered_items.len()
    }

    pub fn get_total_item_count(&self) -> usize {
        self.items.len()
    }
}

/// Mock render context for demonstration
pub struct RenderContext {
    pub width: f32,
    pub height: f32,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockWidget {
        content: String,
    }

    impl Widget for MockWidget {
        fn render(&self, x: f32, y: f32, width: f32, height: f32) {
            println!("Rendering '{}' at ({}, {})", self.content, x, y);
        }

        fn get_height(&self) -> f32 {
            50.0
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    #[test]
    fn test_viewport_manager_visible_range() {
        let config = VirtualizationConfig {
            item_height: 50.0,
            overscan_count: 2,
            estimated_total_height: None,
        };

        let mut manager = ViewportManager::new(&config, 300.0, 100);

        // Initially visible: items 0-8 (300/50 = 6, plus 2 overscan on each side)
        let range = manager.get_visible_range();
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 10); // 6 visible + 4 overscan

        // Scroll down
        manager.set_scroll_top(200.0); // Should show items around index 4
        let range = manager.get_visible_range();
        assert_eq!(range.start, 2); // 4 - 2 overscan
        assert_eq!(range.end, 14); // 4 + 6 + 2 + 2
    }

    #[test]
    fn test_virtualized_list_view() {
        let config = VirtualizationConfig::default();
        let items = (0..1000).collect::<Vec<_>>();

        let mut list_view = VirtualizedListView::new(config, &items, |item| {
            Box::new(MockWidget {
                content: format!("Item {}", item),
            })
        });

        list_view.set_viewport_height(300.0);

        // Initially should render around 6-12 items (depending on overscan)
        assert!(list_view.get_visible_item_count() > 0);
        assert_eq!(list_view.get_total_item_count(), 1000);

        // Scroll should update visible items
        list_view.set_scroll_top(500.0);
        assert!(list_view.get_visible_item_count() > 0);
    }

    #[test]
    fn test_widget_pool() {
        let mut pool = WidgetPool::new(|| {
            Box::new(MockWidget {
                content: "pooled".to_string(),
            })
        });

        // Get a widget
        let widget1 = pool.get();
        assert_eq!(widget1.content, "pooled");

        // Return it to pool
        pool.recycle(widget1);

        // Get it back
        let widget2 = pool.get();
        assert_eq!(widget2.content, "pooled");
    }
}