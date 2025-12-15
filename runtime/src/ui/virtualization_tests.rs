//! Tests for virtualized list view implementation

use super::virtualization::*;

struct TestWidget {
    id: usize,
    rendered_positions: std::cell::RefCell<Vec<(f32, f32)>>,
}

impl TestWidget {
    fn new(id: usize) -> Self {
        Self {
            id,
            rendered_positions: std::cell::RefCell::new(Vec::new()),
        }
    }
}

impl Widget for TestWidget {
    fn render(&self, x: f32, y: f32, width: f32, height: f32) {
        self.rendered_positions.borrow_mut().push((x, y));
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
    fn test_viewport_manager_basic() {
        let config = VirtualizationConfig {
            item_height: 50.0,
            overscan_count: 2,
            estimated_total_height: None,
        };

        let manager = ViewportManager::new(&config, 300.0, 100);

        // Test initial visible range
        let range = manager.get_visible_range();
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 10); // 6 visible + 4 overscan

        // Test total height
        assert_eq!(manager.get_total_height(), 5000.0); // 100 * 50

        // Test item offset
        assert_eq!(manager.get_item_offset(5), 250.0); // 5 * 50
    }

    #[test]
    fn test_viewport_manager_scrolling() {
        let config = VirtualizationConfig {
            item_height: 50.0,
            overscan_count: 1,
            estimated_total_height: None,
        };

        let mut manager = ViewportManager::new(&config, 200.0, 100);

        // Scroll to show items around index 10
        manager.set_scroll_top(500.0); // 500 / 50 = 10
        let range = manager.get_visible_range();

        // Should show items 9-15 (10 ± 4 + overscan)
        assert_eq!(range.start, 9);
        assert_eq!(range.end, 15);
    }

    #[test]
    fn test_viewport_manager_edge_cases() {
        let config = VirtualizationConfig::default();
        let manager = ViewportManager::new(&config, 100.0, 0);

        // Empty list
        let range = manager.get_visible_range();
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 0);
        assert!(range.is_empty());

        // Single item
        let manager = ViewportManager::new(&config, 100.0, 1);
        let range = manager.get_visible_range();
        assert_eq!(range.start, 0);
        assert_eq!(range.end, 1);
    }

    #[test]
    fn test_widget_pool() {
        let mut pool = WidgetPool::new(|| Box::new(TestWidget::new(0)));

        // Get first widget
        let widget1 = pool.get();
        assert_eq!(widget1.id, 0);

        // Recycle it
        pool.recycle(widget1);

        // Get it back
        let widget2 = pool.get();
        assert_eq!(widget2.id, 0);

        // Pool should be empty again
        assert_eq!(pool.available.len(), 0);
    }

    #[test]
    fn test_virtualized_list_view_initialization() {
        let config = VirtualizationConfig {
            item_height: 50.0,
            overscan_count: 2,
            estimated_total_height: None,
        };

        let items = (0..100).collect::<Vec<_>>();
        let list_view = VirtualizedListView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        assert_eq!(list_view.get_total_item_count(), 100);
        assert_eq!(list_view.get_total_height(), 5000.0);
    }

    #[test]
    fn test_virtualized_list_view_scrolling() {
        let config = VirtualizationConfig {
            item_height: 50.0,
            overscan_count: 1,
            estimated_total_height: None,
        };

        let items = (0..1000).collect::<Vec<_>>();
        let mut list_view = VirtualizedListView::new(config.clone(), &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        list_view.set_viewport_height(300.0);

        // Initially should have some visible items
        let initial_count = list_view.get_visible_item_count();
        assert!(initial_count > 0);

        // Scroll down - should update visible items
        list_view.set_scroll_top(1000.0); // Scroll to show items around index 20

        // Should still have visible items, but different ones
        let scrolled_count = list_view.get_visible_item_count();
        assert!(scrolled_count > 0);
    }

    #[test]
    fn test_virtualized_list_view_memory_efficiency() {
        let config = VirtualizationConfig {
            item_height: 50.0,
            overscan_count: 2,
            estimated_total_height: None,
        };

        // Create a large list
        let items = (0..10000).collect::<Vec<_>>();
        let mut list_view = VirtualizedListView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        list_view.set_viewport_height(500.0); // Shows ~10 items

        // Even with 10,000 items, should only render visible ones
        let visible_count = list_view.get_visible_item_count();
        assert!(visible_count <= 20); // 10 visible + overscan

        // Memory usage should be constant regardless of total items
        assert!(visible_count < 100); // Much less than total
    }

    #[test]
    fn test_virtualized_list_view_rendering() {
        let config = VirtualizationConfig::default();
        let items = vec![1, 2, 3, 4, 5];

        let mut list_view = VirtualizedListView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        list_view.set_viewport_height(100.0);

        let context = RenderContext {
            width: 400.0,
            height: 100.0,
        };

        // Render should work without panicking
        list_view.render(&context);

        // Check that widgets were rendered at correct positions
        for (index, widget) in &list_view.rendered_items {
            // Cast to TestWidget to access rendered_positions
            if let Some(test_widget) = widget.as_any().downcast_ref::<TestWidget>() {
                let positions = test_widget.rendered_positions.borrow();
                assert!(!positions.is_empty(), "Widget {} should have been rendered", index);
            }
        }
    }

    #[test]
    fn test_performance_large_list() {
        let config = VirtualizationConfig {
            item_height: 20.0,
            overscan_count: 5,
            estimated_total_height: None,
        };

        // Simulate a very large list (1 million items)
        let items = (0..1_000_000).collect::<Vec<_>>();
        let mut list_view = VirtualizedListView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        list_view.set_viewport_height(400.0); // Shows ~20 items

        // Should only render a small subset
        let visible_count = list_view.get_visible_item_count();
        assert!(visible_count <= 30); // 20 + overscan

        // Performance test: scrolling through the list
        for scroll_pos in (0..100_000).step_by(10_000) {
            list_view.set_scroll_top(scroll_pos as f32);
            assert!(list_view.get_visible_item_count() <= 30);
        }
    }

    #[test]
    fn test_different_item_heights() {
        let config = VirtualizationConfig {
            item_height: 100.0, // Larger items
            overscan_count: 1,
            estimated_total_height: None,
        };

        let items = (0..100).collect::<Vec<_>>();
        let mut list_view = VirtualizedListView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        list_view.set_viewport_height(250.0); // Shows ~2.5 items

        let visible_count = list_view.get_visible_item_count();
        assert!(visible_count <= 6); // 2-3 visible + overscan
    }

    #[test]
    fn test_grid_viewport_manager_basic() {
        let config = GridVirtualizationConfig {
            item_width: 100.0,
            item_height: 100.0,
            columns: 3,
            overscan_count: 1,
            max_pool_size: 20,
        };

        let manager = GridViewportManager::new(&config, 300.0, 200.0, 100);

        // Test initial visible range
        let range = manager.get_visible_range();
        assert_eq!(range.start_item, 0);
        assert_eq!(range.end_item, 12); // 2 rows * 3 cols + overscan

        // Test total size
        let (width, height) = manager.get_total_size();
        assert_eq!(width, 300.0); // 3 columns * 100
        assert_eq!(height, 3400.0); // ~34 rows * 100
    }

    #[test]
    fn test_grid_viewport_manager_scrolling() {
        let config = GridVirtualizationConfig {
            item_width: 100.0,
            item_height: 100.0,
            columns: 4,
            overscan_count: 1,
            max_pool_size: 20,
        };

        let mut manager = GridViewportManager::new(&config, 400.0, 200.0, 100);

        // Scroll down and right
        manager.set_scroll_position(100.0, 200.0);
        let range = manager.get_visible_range();

        // Should show items from row 2-4, cols 1-4
        assert!(range.start_item >= 8); // Row 2 * 4 cols
        assert!(range.end_item <= 24); // Row 4 * 4 cols
    }

    #[test]
    fn test_grid_viewport_manager_item_position() {
        let config = GridVirtualizationConfig {
            item_width: 50.0,
            item_height: 50.0,
            columns: 5,
            overscan_count: 1,
            max_pool_size: 20,
        };

        let manager = GridViewportManager::new(&config, 250.0, 100.0, 100);

        // Item at index 7 (row 1, col 2)
        let (x, y) = manager.get_item_position(7);
        assert_eq!(x, 100.0); // col 2 * 50
        assert_eq!(y, 50.0);  // row 1 * 50

        // Item at index 15 (row 3, col 0)
        let (x, y) = manager.get_item_position(15);
        assert_eq!(x, 0.0);   // col 0 * 50
        assert_eq!(y, 150.0); // row 3 * 50
    }

    #[test]
    fn test_virtualized_grid_view_initialization() {
        let config = GridVirtualizationConfig {
            item_width: 100.0,
            item_height: 100.0,
            columns: 3,
            overscan_count: 2,
            max_pool_size: 20,
        };

        let items = (0..50).collect::<Vec<_>>();
        let grid_view = VirtualizedGridView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        let (width, height) = grid_view.get_total_size();
        assert_eq!(width, 300.0); // 3 columns * 100
        assert_eq!(height, 1700.0); // ~17 rows * 100
    }

    #[test]
    fn test_virtualized_grid_view_scrolling() {
        let config = GridVirtualizationConfig {
            item_width: 100.0,
            item_height: 100.0,
            columns: 4,
            overscan_count: 1,
            max_pool_size: 30,
        };

        let items = (0..100).collect::<Vec<_>>();
        let mut grid_view = VirtualizedGridView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        // Initially should have some rendered items
        grid_view.update_visible_items();
        let initial_count = grid_view.rendered_items.len();
        assert!(initial_count > 0);

        // Scroll to different position
        grid_view.set_scroll_position(200.0, 300.0);
        let scrolled_count = grid_view.rendered_items.len();
        assert!(scrolled_count > 0);

        // Should have different items rendered
        // (This is a basic check - in real implementation we'd verify specific indices)
    }

    #[test]
    fn test_grid_view_memory_efficiency() {
        let config = GridVirtualizationConfig {
            item_width: 50.0,
            item_height: 50.0,
            columns: 10,
            overscan_count: 2,
            max_pool_size: 50,
        };

        // Large grid: 1000 items in 10x10 layout
        let items = (0..1000).collect::<Vec<_>>();
        let mut grid_view = VirtualizedGridView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        grid_view.update_visible_items();

        // With viewport of 800x600, should only render visible portion
        // 800/50 = 16 cols, 600/50 = 12 rows = ~192 items visible
        // With overscan: ~300 items max
        let rendered_count = grid_view.rendered_items.len();
        assert!(rendered_count <= 400); // Much less than 1000 total
        assert!(rendered_count > 0);
    }
}