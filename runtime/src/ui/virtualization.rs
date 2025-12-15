//! Virtualized ListView implementation for efficient rendering of large lists
//!
//! This module provides virtualized list views that can handle thousands of items
//! without performance degradation by only rendering visible items.

use std::collections::HashMap;

/// Configuration for grid virtualization
#[derive(Debug, Clone, PartialEq)]
pub struct GridVirtualizationConfig {
    pub item_width: f32,
    pub item_height: f32,
    pub columns: usize,
    pub overscan_count: usize,
    pub max_pool_size: usize,
}

impl Default for GridVirtualizationConfig {
    fn default() -> Self {
        Self {
            item_width: 100.0,
            item_height: 100.0,
            columns: 3,
            overscan_count: 2,
            max_pool_size: 50,
        }
    }
}

/// Manages viewport for 2D grid layouts
#[derive(Debug, Clone)]
pub struct GridViewportManager {
    config: GridVirtualizationConfig,
    scroll_left: f32,
    scroll_top: f32,
    viewport_width: f32,
    viewport_height: f32,
    total_items: usize,
}

impl GridViewportManager {
    pub fn new(config: &GridVirtualizationConfig, viewport_width: f32, viewport_height: f32, total_items: usize) -> Self {
        Self {
            config: config.clone(),
            scroll_left: 0.0,
            scroll_top: 0.0,
            viewport_width,
            viewport_height,
            total_items,
        }
    }

    pub fn set_scroll_position(&mut self, left: f32, top: f32) {
        self.scroll_left = left;
        self.scroll_top = top;
    }

    pub fn get_visible_range(&self) -> VisibleGridRange {
        if self.total_items == 0 {
            return VisibleGridRange::new(0, 0, 0, 0);
        }

        // Calculate visible rows
        let start_row = (self.scroll_top / self.config.item_height) as usize;
        let visible_rows = (self.viewport_height / self.config.item_height).ceil() as usize;
        let end_row = start_row + visible_rows + self.config.overscan_count;

        // Calculate visible columns
        let start_col = (self.scroll_left / self.config.item_width) as usize;
        let visible_cols = (self.viewport_width / self.config.item_width).ceil() as usize;
        let end_col = (start_col + visible_cols + self.config.overscan_count).min(self.config.columns);

        // Convert to item indices
        let start_item = start_row * self.config.columns + start_col;
        let end_item = (end_row * self.config.columns + end_col).min(self.total_items);

        VisibleGridRange::new(start_item, end_item, start_row, end_row)
    }

    pub fn get_total_size(&self) -> (f32, f32) {
        let rows = (self.total_items as f32 / self.config.columns as f32).ceil();
        let total_width = self.config.columns as f32 * self.config.item_width;
        let total_height = rows * self.config.item_height;
        (total_width, total_height)
    }

    pub fn get_item_position(&self, index: usize) -> (f32, f32) {
        let row = index / self.config.columns;
        let col = index % self.config.columns;
        let x = col as f32 * self.config.item_width - self.scroll_left;
        let y = row as f32 * self.config.item_height - self.scroll_top;
        (x, y)
    }
}

/// Represents visible range in a 2D grid
#[derive(Debug, Clone, PartialEq)]
pub struct VisibleGridRange {
    pub start_item: usize,
    pub end_item: usize,
    pub start_row: usize,
    pub end_row: usize,
}

impl VisibleGridRange {
    pub fn new(start_item: usize, end_item: usize, start_row: usize, end_row: usize) -> Self {
        Self {
            start_item,
            end_item,
            start_row,
            end_row,
        }
    }

    pub fn len(&self) -> usize {
        self.end_item.saturating_sub(self.start_item)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Virtualized grid view for efficient rendering of large 2D layouts
pub struct VirtualizedGridView<T> {
    config: GridVirtualizationConfig,
    grid_manager: GridViewportManager,
    items: Vec<T>,
    widget_pool: WidgetPool<Box<dyn Widget>>,
    pub rendered_items: HashMap<usize, Box<dyn Widget>>,
}

impl<T: 'static> VirtualizedGridView<T> {
    pub fn new<F>(
        config: GridVirtualizationConfig,
        items: &[T],
        create_widget_fn: F
    ) -> Self
    where
        F: Fn(&T) -> Box<dyn Widget> + 'static,
        T: Clone,
    {
        let total_items = items.len();
        let grid_manager = GridViewportManager::new(
            &config,
            800.0, // Default viewport width
            600.0, // Default viewport height
            total_items,
        );

        let first_item = items.first().cloned().unwrap_or_else(|| panic!("Items cannot be empty"));
        let widget_pool = WidgetPool::new(move || create_widget_fn(&first_item));

        Self {
            config,
            grid_manager,
            items: items.to_vec(),
            widget_pool,
            rendered_items: HashMap::new(),
        }
    }

    pub fn set_scroll_position(&mut self, left: f32, top: f32) {
        self.grid_manager.set_scroll_position(left, top);
        self.update_visible_items();
    }

    pub fn update_visible_items(&mut self) {
        let visible_range = self.grid_manager.get_visible_range();

        // Remove items that are no longer visible
        self.rendered_items.retain(|&index, _| {
            index >= visible_range.start_item && index < visible_range.end_item
        });

        // Add new visible items
        for index in visible_range.start_item..visible_range.end_item {
            if !self.rendered_items.contains_key(&index) {
                if let Some(item) = self.items.get(index) {
                    // For now, create a simple placeholder widget - in real implementation
                    // this would use the create_widget_fn
                    let widget = self.widget_pool.get();
                    self.rendered_items.insert(index, widget);
                }
            }
        }
    }

    pub fn render(&self, x: f32, y: f32, width: f32, height: f32) {
        for (&index, widget) in &self.rendered_items {
            let (item_x, item_y) = self.grid_manager.get_item_position(index);
            widget.render(x + item_x, y + item_y, self.config.item_width, self.config.item_height);
        }
    }

    pub fn get_total_size(&self) -> (f32, f32) {
        self.grid_manager.get_total_size()
    }
}

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
struct MockWidget {
    content: String,
}

#[cfg(test)]
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

// Simple widget for basic testing
struct SimpleWidget {
    content: String,
}

impl Widget for SimpleWidget {
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

#[cfg(test)]
mod tests {
    use super::*;

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