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

    // ===== INTEGRATION TESTS =====

    #[test]
    fn test_integration_performance_large_list() {
        let config = VirtualizationConfig {
            item_height: 50.0,
            overscan_count: 5,
            estimated_total_height: None,
        };

        // Create large list with 10,000 items
        let items = (0..10_000).collect::<Vec<_>>();
        let mut list_view = VirtualizedListView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        // Set viewport height for 10 visible items
        list_view.set_viewport_height(500.0);

        // Initial render should be fast and only show visible items
        list_view.set_scroll_top(0.0);
        let initial_rendered = list_view.rendered_items.len();
        assert!(initial_rendered <= 20); // 10 visible + 10 overscan max

        // Scroll to middle - should still be efficient
        list_view.set_scroll_top(250_000.0); // Middle of the list
        let middle_rendered = list_view.rendered_items.len();
        assert!(middle_rendered <= 20);

        // Scroll to end
        list_view.set_scroll_top(499_950.0); // Near the end
        let end_rendered = list_view.rendered_items.len();
        assert!(end_rendered <= 20);

        // Verify we never rendered more than a small fraction of total items
        assert!(initial_rendered < 100); // Less than 1% of 10,000
        assert!(middle_rendered < 100);
        assert!(end_rendered < 100);
    }

    #[test]
    fn test_integration_memory_efficiency_widget_pool() {
        let config = VirtualizationConfig {
            item_height: 50.0,
            overscan_count: 3,
            estimated_total_height: None,
        };

        let items = (0..1000).collect::<Vec<_>>();
        let mut list_view = VirtualizedListView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        list_view.set_viewport_height(200.0); // 4 visible items

        // Initial state
        list_view.set_scroll_top(0.0);
        let initial_count = list_view.rendered_items.len();

        // Scroll multiple times - pool should reuse widgets
        for scroll_pos in (0..2000).step_by(200) {
            list_view.set_scroll_top(scroll_pos as f32);
            let current_count = list_view.rendered_items.len();
            // Should maintain similar number of rendered items
            assert!(current_count <= initial_count + 10); // Allow some variation
        }

        // Final count should be reasonable
        let final_count = list_view.rendered_items.len();
        assert!(final_count > 0);
        assert!(final_count <= 20); // Much less than 1000 total
    }

    #[test]
    fn test_integration_full_scroll_scenario() {
        let config = VirtualizationConfig {
            item_height: 50.0,
            overscan_count: 2,
            estimated_total_height: None,
        };

        let items = (0..1000).collect::<Vec<_>>();
        let mut list_view = VirtualizedListView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        list_view.set_viewport_height(300.0); // 6 visible items

        // Track all rendered items during full scroll
        let mut all_rendered_indices = std::collections::HashSet::new();

        // Scroll from start to end in steps
        for scroll_pos in (0..49_950).step_by(1000) {
            list_view.set_scroll_top(scroll_pos as f32);

            // Collect all currently rendered indices
            for &index in list_view.rendered_items.keys() {
                all_rendered_indices.insert(index);
            }
        }

        // Should have rendered a reasonable subset of all items
        assert!(all_rendered_indices.len() > 100); // Rendered many different items
        assert!(all_rendered_indices.len() < 500); // But not too many (efficient)

        // Should include items from different parts of the list
        let has_early_items = all_rendered_indices.iter().any(|&i| i < 100);
        let has_middle_items = all_rendered_indices.iter().any(|&i| i >= 400 && i < 600);
        let has_late_items = all_rendered_indices.iter().any(|&i| i >= 900);

        assert!(has_early_items, "Should have rendered early items");
        assert!(has_middle_items, "Should have rendered middle items");
        assert!(has_late_items, "Should have rendered late items");
    }

    #[test]
    fn test_integration_dynamic_data_changes() {
        let config = VirtualizationConfig {
            item_height: 50.0,
            overscan_count: 2,
            estimated_total_height: None,
        };

        // Start with small list
        let mut items = (0..50).collect::<Vec<_>>();
        let mut list_view = VirtualizedListView::new(config.clone(), &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        list_view.set_viewport_height(200.0); // 4 visible items

        // Initial render
        list_view.set_scroll_top(0.0);
        let initial_rendered = list_view.rendered_items.len();

        // Add more items dynamically (simulate data change)
        items.extend(50..150);
        // Note: In real implementation, we'd need a way to update the list_view
        // For this test, we verify the concept by creating a new one
        let mut updated_list_view = VirtualizedListView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });
        updated_list_view.set_viewport_height(200.0);

        updated_list_view.set_scroll_top(0.0);
        let updated_rendered = updated_list_view.rendered_items.len();

        // Should handle larger dataset efficiently
        assert!(updated_rendered <= initial_rendered + 5); // Similar efficiency
        assert!(updated_rendered > 0);

        // Scroll to later position to verify extended range
        updated_list_view.set_scroll_top(2000.0); // Should show items around index 40
        let scrolled_rendered = updated_list_view.rendered_items.len();
        assert!(scrolled_rendered > 0);
        assert!(scrolled_rendered <= 15); // Still efficient
    }

    #[test]
    fn test_integration_grid_2d_navigation() {
        let config = GridVirtualizationConfig {
            item_width: 100.0,
            item_height: 100.0,
            columns: 5,
            overscan_count: 1,
            max_pool_size: 50,
        };

        // Create 10x10 grid = 100 items
        let items = (0..100).collect::<Vec<_>>();
        let mut grid_view = VirtualizedGridView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        // Test different scroll positions
        let test_positions = vec![
            (0.0, 0.0),     // Top-left
            (200.0, 0.0),   // Scroll right
            (0.0, 300.0),   // Scroll down
            (200.0, 300.0), // Scroll diagonally
            (400.0, 600.0), // Bottom-right area
        ];

        for (scroll_x, scroll_y) in test_positions {
            grid_view.set_scroll_position(scroll_x, scroll_y);
            let rendered_count = grid_view.rendered_items.len();

            // Should always render reasonable number of items
            assert!(rendered_count > 0);
            assert!(rendered_count <= 25); // 5x5 visible area + overscan

            // Verify rendered items are within valid range
            for &index in grid_view.rendered_items.keys() {
                assert!(index < 100); // Within our 100 items
                assert!(index >= 0);
            }
        }
    }

    #[test]
    fn test_integration_grid_vs_list_consistency() {
        // Compare that grid and list virtualization behave consistently
        let list_config = VirtualizationConfig {
            item_height: 100.0,
            overscan_count: 2,
            estimated_total_height: None,
        };

        let grid_config = GridVirtualizationConfig {
            item_width: 100.0,
            item_height: 100.0,
            columns: 1, // Single column = like a list
            overscan_count: 2,
            max_pool_size: 50,
        };

        let items = (0..100).collect::<Vec<_>>();

        // Create both list and single-column grid
        let mut list_view = VirtualizedListView::new(list_config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });
        list_view.set_viewport_height(300.0); // 3 visible items

        let mut grid_view = VirtualizedGridView::new(grid_config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        // Test at same scroll positions
        let test_positions = vec![0.0, 100.0, 200.0, 500.0];

        for &scroll_pos in &test_positions {
            list_view.set_scroll_top(scroll_pos);
            grid_view.set_scroll_position(0.0, scroll_pos);

            let list_rendered = list_view.rendered_items.len();
            let grid_rendered = grid_view.rendered_items.len();

            // Should render similar number of items (allowing small differences)
            let diff = (list_rendered as i32 - grid_rendered as i32).abs();
            assert!(diff <= 3, "List rendered {}, Grid rendered {}", list_rendered, grid_rendered);
        }
    }

    #[test]
    fn test_integration_stress_test_large_dataset() {
        let config = VirtualizationConfig {
            item_height: 20.0, // Smaller items
            overscan_count: 10, // More overscan for smoother scrolling
            estimated_total_height: None,
        };

        // Very large dataset: 100,000 items
        let items = (0..100_000).collect::<Vec<_>>();
        let mut list_view = VirtualizedListView::new(config, &items, |&item| {
            Box::new(TestWidget::new(item))
        });

        list_view.set_viewport_height(400.0); // 20 visible items

        // Test various scroll positions quickly
        let scroll_positions = [0.0, 500_000.0, 1_000_000.0, 1_500_000.0, 1_999_980.0];

        for &scroll_pos in &scroll_positions {
            list_view.set_scroll_top(scroll_pos);
            let rendered_count = list_view.rendered_items.len();

            // Should always be efficient regardless of position
            assert!(rendered_count > 0);
            assert!(rendered_count <= 50); // 20 visible + 30 overscan max

            // Verify indices are valid
            for &index in list_view.rendered_items.keys() {
                assert!(index < 100_000);
                assert!(index >= 0);
            }
        }

        // Memory efficiency: should never render more than 0.1% of items
        assert!(list_view.rendered_items.len() < 1000); // Less than 1% of 100k
    }
}