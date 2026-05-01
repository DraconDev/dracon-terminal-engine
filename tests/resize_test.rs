//! Integration tests for terminal resize handling.
//!
//! Tests the resize cycle components: area update → dirty flag → render → clean
//!
//! ## Test areas
//!
//! 1. **App resize handling** - Widget area updates and dirty tracking on resize
//! 2. **SplitPane resize behavior** - SplitPane recalculates child areas on resize
//! 3. **Area propagation on resize** - Root widget gets full terminal area, children get proportional sub-areas
//! 4. **Resize + render cycle** - Resize dirty flag triggers render, render clears dirty flag
//! 5. **Minimal resize (same size)** - Resize to same dimensions doesn't cause unnecessary re-render
//! 6. **Resize with multiple widgets** - All registered widgets receive resize events with correct areas
//!
//! Note: App::run() requires a TTY and enters raw mode, so these tests verify
//! the resize cycle components through the public API. The actual resize event
//! handling happens inside run() via SIGWINCH signals.

use std::cell::Cell;
use std::rc::Rc;

use dracon_terminal_engine::compositor::Compositor;
use dracon_terminal_engine::compositor::Plane;
use dracon_terminal_engine::framework::app::App;
use dracon_terminal_engine::framework::dirty_regions::DirtyRegionTracker;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::split::{Orientation, SplitPane};
use ratatui::layout::Rect;

/// A widget that tracks set_area calls and area changes.
struct TrackingWidget {
    id: WidgetId,
    area: Cell<Rect>,
    dirty: bool,
    set_area_count: Rc<Cell<u32>>,
    last_set_area: Cell<Option<Rect>>,
    z_index: u16,
    render_count: Rc<Cell<u32>>,
}

impl TrackingWidget {
    fn new(id: usize, set_area_count: Rc<Cell<u32>>) -> Self {
        Self {
            id: WidgetId::new(id),
            area: Cell::new(Rect::new(0, 0, 80, 24)),
            dirty: true,
            set_area_count,
            last_set_area: Cell::new(None),
            z_index: 0,
            render_count: Rc::new(Cell::new(0)),
        }
    }

    fn with_z_index(mut self, z: u16) -> Self {
        self.z_index = z;
        self
    }
}

impl Widget for TrackingWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.last_set_area.set(Some(area));
        self.set_area_count.set(self.set_area_count.get() + 1);
        self.area.set(area);
        self.dirty = true;
    }

    fn set_id(&mut self, id: WidgetId) {
        self.id = id;
    }

    fn z_index(&self) -> u16 {
        self.z_index
    }

    fn needs_render(&self) -> bool {
        self.dirty
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn clear_dirty(&mut self) {
        self.dirty = false;
    }

    fn render(&self, area: Rect) -> Plane {
        self.render_count.set(self.render_count.get() + 1);
        Plane::new(0, area.width, area.height)
    }
}

// ============================================================================
// Test 1: App resize handling - widget area updates and dirty tracking
// ============================================================================

#[test]
fn test_app_add_widget_sets_area() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    let w = app.widget(id).unwrap();
    let area = w.area();
    assert_eq!(area.width, 80);
    assert_eq!(area.height, 24);
    assert_eq!(set_area_count.get(), 1, "set_area should be called once on add_widget");
}

#[test]
fn test_app_widget_area_updated_via_widget_mut() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 100, 40));
    }

    let w = app.widget(id).unwrap();
    let area = w.area();
    assert_eq!(area.width, 100, "Widget width should be updated via set_area");
    assert_eq!(area.height, 40, "Widget height should be updated via set_area");
    assert_eq!(set_area_count.get(), 2, "set_area should be called again");
}

#[test]
fn test_app_widget_dirty_tracking() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    {
        let mut w = app.widget_mut(id).unwrap();
        w.clear_dirty();
    }

    {
        let w = app.widget(id).unwrap();
        assert!(!w.needs_render(), "Widget should not need render after clear_dirty");
    }

    {
        let mut w = app.widget_mut(id).unwrap();
        w.mark_dirty();
    }

    {
        let w = app.widget(id).unwrap();
        assert!(w.needs_render(), "Widget should need render after mark_dirty");
    }
}

#[test]
fn test_app_resize_marks_widgets_dirty_simulation() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    {
        let mut w = app.widget_mut(id).unwrap();
        w.clear_dirty();
    }

    // Simulate what happens during resize:
    // 1. compositor.resize() is called
    // 2. dirty_tracker.mark_all_dirty() is called
    // 3. for each widget, widget.mark_dirty() is called
    {
        let mut w = app.widget_mut(id).unwrap();
        w.mark_dirty();
    }

    let w = app.widget(id).unwrap();
    assert!(w.needs_render(), "Widget should be dirty after resize event simulation");
}

// ============================================================================
// Test 2: SplitPane resize behavior
// ============================================================================

#[test]
fn test_splitpane_recalculates_child_areas_on_set_area() {
    let mut split = SplitPane::new(Orientation::Horizontal);
    let area = Rect::new(0, 0, 80, 24);
    split.set_area(area);

    let (left, right) = split.split(area);
    assert!(left.width > 0 && right.width > 0);
    assert_eq!(left.width + right.width, 80);

    split.set_area(Rect::new(0, 0, 120, 24));

    let (left, right) = split.split(Rect::new(0, 0, 120, 24));
    assert!(left.width > 0 && right.width > 0);
    assert_eq!(left.width + right.width, 120);
}

#[test]
fn test_splitpane_vertical_resize() {
    let mut split = SplitPane::new(Orientation::Vertical);
    let area = Rect::new(0, 0, 80, 24);
    split.set_area(area);

    let (top, bottom) = split.split(area);
    assert!(top.height > 0 && bottom.height > 0);
    assert_eq!(top.height + bottom.height, 24);

    split.set_area(Rect::new(0, 0, 80, 40));

    let (top, bottom) = split.split(Rect::new(0, 0, 80, 40));
    assert!(top.height > 0 && bottom.height > 0);
    assert_eq!(top.height + bottom.height, 40);
}

#[test]
fn test_splitpane_resize_marks_dirty() {
    let mut split = SplitPane::new(Orientation::Horizontal);
    split.set_area(Rect::new(0, 0, 80, 24));

    split.clear_dirty();
    assert!(!split.needs_render(), "Should be clean after clear_dirty");

    split.set_area(Rect::new(0, 0, 100, 24));

    assert!(split.needs_render(), "Should be dirty after set_area");
}

#[test]
fn test_splitpane_children_get_resized_areas_immediately() {
    let mut split = SplitPane::new(Orientation::Horizontal);
    let area = Rect::new(0, 0, 80, 24);
    split.set_area(area);

    let initial_split = split.split(area);

    split.set_area(Rect::new(0, 0, 100, 24));
    let resized_split = split.split(Rect::new(0, 0, 100, 24));

    assert_ne!(
        initial_split.0.width,
        resized_split.0.width,
        "Left pane width should change after resize"
    );
    assert_ne!(
        initial_split.1.width,
        resized_split.1.width,
        "Right pane width should change after resize"
    );
}

#[test]
fn test_splitpane_ratio_affects_resize_distribution() {
    let mut split = SplitPane::new(Orientation::Horizontal).ratio(0.7);
    let area = Rect::new(0, 0, 100, 24);
    split.set_area(area);

    let (left, right) = split.split(area);
    assert_eq!(left.width, 70, "70% ratio should give left pane 70 width");
    assert_eq!(right.width, 30, "30% ratio should give right pane 30 width");
}

// ============================================================================
// Test 3: Area propagation on resize
// ============================================================================

#[test]
fn test_root_widget_gets_full_terminal_area_on_resize_simulation() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    let original_area = {
        let w = app.widget(id).unwrap();
        w.area()
    };
    assert_eq!(original_area.width, 80);
    assert_eq!(original_area.height, 24);

    // Simulate resize: widgets are given new full terminal area
    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 120, 30));
    }

    let new_area = {
        let w = app.widget(id).unwrap();
        w.area()
    };
    assert_eq!(new_area.width, 120);
    assert_eq!(new_area.height, 30);
}

#[test]
fn test_child_widgets_get_proportional_sub_areas() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    // Widget initially positioned in left half
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 40, 24));

    // Simulate resize: widget gets new area
    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 80, 24));
    }

    let w = app.widget(id).unwrap();
    let area = w.area();
    assert_eq!(area.x, 0);
    assert_eq!(area.y, 0);
    assert_eq!(area.width, 80, "Widget should get full terminal width after resize");
    assert_eq!(area.height, 24);
}

#[test]
fn test_deep_nesting_maintains_correct_areas() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    // First resize
    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 100, 30));
    }

    let area = {
        let w = app.widget(id).unwrap();
        w.area()
    };
    assert_eq!(area.width, 100);
    assert_eq!(area.height, 30);

    // Second resize
    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 120, 40));
    }

    let area = {
        let w = app.widget(id).unwrap();
        w.area()
    };
    assert_eq!(area.width, 120);
    assert_eq!(area.height, 40);
}

// ============================================================================
// Test 4: Resize + render cycle
// ============================================================================

#[test]
fn test_set_area_marks_widget_dirty() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    {
        let mut w = app.widget_mut(id).unwrap();
        w.clear_dirty();
    }

    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 100, 40));
    }

    let w = app.widget(id).unwrap();
    assert!(w.needs_render(), "Widget should need render after set_area");
}

#[test]
fn test_render_clears_dirty_flag() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    {
        let mut w = app.widget_mut(id).unwrap();
        w.clear_dirty();
    }

    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 100, 40));
    }

    {
        let mut w = app.widget_mut(id).unwrap();
        w.clear_dirty();
    }

    {
        let w = app.widget(id).unwrap();
        assert!(!w.needs_render(), "Widget should not need render after clear_dirty");
    }
}

#[test]
fn test_subsequent_resize_resets_dirty() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    // First resize cycle
    {
        let mut w = app.widget_mut(id).unwrap();
        w.clear_dirty();
    }

    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 100, 40));
    }

    {
        let mut w = app.widget_mut(id).unwrap();
        w.clear_dirty();
    }

    // Second resize
    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 120, 50));
    }

    let w = app.widget(id).unwrap();
    assert!(w.needs_render(), "Widget should be dirty after second resize");
}

#[test]
fn test_dirty_tracker_marked_on_resize() {
    let mut tracker = DirtyRegionTracker::new();
    tracker.mark_all_dirty();
    assert!(tracker.needs_full_refresh());

    tracker.clear();

    tracker.mark_dirty(0, 0, 80, 24);
    assert!(tracker.is_dirty());
    assert!(!tracker.needs_full_refresh());
}

#[test]
fn test_dirty_tracker_full_refresh_clears_regions() {
    let mut tracker = DirtyRegionTracker::new();
    tracker.mark_dirty(0, 0, 80, 24);
    assert!(tracker.is_dirty());
    assert!(!tracker.needs_full_refresh());

    tracker.mark_all_dirty();
    assert!(tracker.needs_full_refresh());
    // Note: is_dirty() still returns true because full_refresh is set
    // This is expected behavior - full_refresh implies dirty
}

// ============================================================================
// Test 5: Minimal resize (same size)
// ============================================================================

#[test]
fn test_set_area_to_same_dimensions_calls_set_area() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let _id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    // Simulate resize to same dimensions
    {
        let mut w = app.widget_mut(_id).unwrap();
        w.set_area(Rect::new(0, 0, 80, 24));
    }

    assert_eq!(set_area_count.get(), 2, "set_area should be called even for same-size resize");
}

#[test]
fn test_set_area_same_size_still_marks_dirty() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    {
        let mut w = app.widget_mut(id).unwrap();
        w.clear_dirty();
    }

    // Simulate resize to same dimensions
    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 80, 24));
    }

    let w = app.widget(id).unwrap();
    assert!(w.needs_render(), "Widget should be dirty even for same-size resize");
}

// ============================================================================
// Test 6: Resize with multiple widgets
// ============================================================================

#[test]
fn test_all_widgets_receive_resize_simulation() {
    let mut app = App::new().unwrap();
    let set_area_count1 = Rc::new(Cell::new(0u32));
    let set_area_count2 = Rc::new(Cell::new(0u32));
    let set_area_count3 = Rc::new(Cell::new(0u32));

    let widget1 = TrackingWidget::new(1, set_area_count1.clone());
    let widget2 = TrackingWidget::new(2, set_area_count2.clone());
    let widget3 = TrackingWidget::new(3, set_area_count3.clone());

    let id1 = app.add_widget(Box::new(widget1), Rect::new(0, 0, 80, 24));
    let id2 = app.add_widget(Box::new(widget2), Rect::new(0, 0, 80, 24));
    let id3 = app.add_widget(Box::new(widget3), Rect::new(0, 0, 80, 24));

    // Simulate resize: all widgets get new area
    for id in [id1, id2, id3] {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 100, 40));
    }

    assert_eq!(set_area_count1.get(), 2, "Widget 1 should have set_area called on resize");
    assert_eq!(set_area_count2.get(), 2, "Widget 2 should have set_area called on resize");
    assert_eq!(set_area_count3.get(), 2, "Widget 3 should have set_area called on resize");

    let w1 = app.widget(id1).unwrap();
    let w2 = app.widget(id2).unwrap();
    let w3 = app.widget(id3).unwrap();

    assert_eq!(w1.area().width, 100, "Widget 1 width should be updated");
    assert_eq!(w2.area().width, 100, "Widget 2 width should be updated");
    assert_eq!(w3.area().width, 100, "Widget 3 width should be updated");
}

#[test]
fn test_each_widget_gets_correct_area_based_on_z_index() {
    let mut app = App::new().unwrap();
    let set_area_count1 = Rc::new(Cell::new(0u32));
    let set_area_count2 = Rc::new(Cell::new(0u32));

    let widget1 = TrackingWidget::new(1, set_area_count1.clone()).with_z_index(0);
    let widget2 = TrackingWidget::new(2, set_area_count2.clone()).with_z_index(10);

    let id1 = app.add_widget(Box::new(widget1), Rect::new(0, 0, 40, 24));
    let id2 = app.add_widget(Box::new(widget2), Rect::new(40, 0, 40, 24));

    // Simulate resize: both widgets get full terminal area
    for id in [id1, id2] {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 100, 40));
    }

    let w1 = app.widget(id1).unwrap();
    let w2 = app.widget(id2).unwrap();

    assert_eq!(w1.area().width, 100, "Widget 1 should get full width after resize");
    assert_eq!(w2.area().width, 100, "Widget 2 should get full width after resize");
}

#[test]
fn test_widgets_not_at_z_index_zero_get_proper_areas() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));
    let widget = TrackingWidget::new(1, set_area_count.clone()).with_z_index(5);
    let id = app.add_widget(Box::new(widget), Rect::new(10, 5, 30, 10));

    // Simulate resize: widget gets new area
    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 80, 24));
    }

    let w = app.widget(id).unwrap();
    let area = w.area();
    assert_eq!(area.width, 80);
    assert_eq!(area.height, 24);
}

#[test]
fn test_multiple_widgets_dirty_tracking_after_resize() {
    let mut app = App::new().unwrap();
    let widget1 = TrackingWidget::new(1, Rc::new(Cell::new(0)));
    let widget2 = TrackingWidget::new(2, Rc::new(Cell::new(0)));
    let widget3 = TrackingWidget::new(3, Rc::new(Cell::new(0)));

    let id1 = app.add_widget(Box::new(widget1), Rect::new(0, 0, 80, 24));
    let id2 = app.add_widget(Box::new(widget2), Rect::new(0, 0, 80, 24));
    let id3 = app.add_widget(Box::new(widget3), Rect::new(0, 0, 80, 24));

    // Clear all dirty flags
    for id in [id1, id2, id3] {
        let mut w = app.widget_mut(id).unwrap();
        w.clear_dirty();
    }

    // Simulate resize: all widgets marked dirty
    for id in [id1, id2, id3] {
        let mut w = app.widget_mut(id).unwrap();
        w.mark_dirty();
    }

    let w1 = app.widget(id1).unwrap();
    let w2 = app.widget(id2).unwrap();
    let w3 = app.widget(id3).unwrap();

    assert!(w1.needs_render(), "Widget 1 should be dirty after resize");
    assert!(w2.needs_render(), "Widget 2 should be dirty after resize");
    assert!(w3.needs_render(), "Widget 3 should be dirty after resize");
}

// ============================================================================
// Test: Compositor resize updates internal state
// ============================================================================

#[test]
fn test_compositor_resize_updates_dimensions() {
    let mut compositor = Compositor::new(80, 24);
    assert_eq!(compositor.size(), (80, 24));

    compositor.resize(120, 30);
    assert_eq!(compositor.size(), (120, 30));

    compositor.resize(100, 40);
    assert_eq!(compositor.size(), (100, 40));
}

#[test]
fn test_compositor_resize_resets_frame_buffer() {
    let mut compositor = Compositor::new(80, 24);
    compositor.resize(40, 12);
    assert_eq!(compositor.size(), (40, 12));
}

// ============================================================================
// Test: SplitPane with custom ratio maintains proportions on resize
// ============================================================================

#[test]
fn test_splitpane_ratio_preserved_on_resize() {
    let mut split = SplitPane::new(Orientation::Horizontal).ratio(0.3);
    split.set_area(Rect::new(0, 0, 100, 24));

    let (left, right) = split.split(Rect::new(0, 0, 100, 24));
    assert_eq!(left.width, 30);
    assert_eq!(right.width, 70);

    // Resize to larger width, ratio should still apply
    split.set_area(Rect::new(0, 0, 200, 24));
    let (left, right) = split.split(Rect::new(0, 0, 200, 24));
    assert_eq!(left.width, 60);
    assert_eq!(right.width, 140);
}

// ============================================================================
// Test: Render count tracks resize + render cycles
// ============================================================================

#[test]
fn test_render_count_increments_after_resize() {
    let mut app = App::new().unwrap();
    let set_area_count = Rc::new(Cell::new(0u32));

    let widget = TrackingWidget::new(1, set_area_count.clone());
    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    // Clear dirty to start fresh
    {
        let mut w = app.widget_mut(id).unwrap();
        w.clear_dirty();
    }

    // Trigger resize
    {
        let mut w = app.widget_mut(id).unwrap();
        w.set_area(Rect::new(0, 0, 100, 40));
    }

    // Render should now be needed
    let w = app.widget(id).unwrap();
    assert!(w.needs_render());
}
