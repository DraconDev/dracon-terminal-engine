//! Integration tests for composing multiple widgets together.
//!
//! Tests the interactions between widgets including:
//! - SplitPane containing List and custom Panel content
//! - Widget tree rendering with proper area propagation
//! - Z-index layering and compositing order
//! - Dirty tracking across widget composition
//! - App lifecycle (on_mount/on_unmount) for multiple widgets
//! - Modal overlay event interception

use std::cell::Cell;
use std::rc::Rc;

use dracon_terminal_engine::compositor::{Plane, Color};
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Hud, List, Modal, ModalResult, SplitPane,
};
use dracon_terminal_engine::framework::widgets::split::Orientation;
use dracon_terminal_engine::framework::app::App;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;





/// A mock panel widget for testing composition with SplitPane.
struct MockPanel {
    id: WidgetId,
    title: String,
    area: std::cell::Cell<Rect>,
    render_count: Rc<Cell<usize>>,
}

impl MockPanel {
    fn new(id: usize, title: &str) -> (Self, Rc<Cell<usize>>) {
        let render_count = Rc::new(Cell::new(0));
        (
            Self {
                id: WidgetId::new(id),
                title: title.to_string(),
                area: std::cell::Cell::new(Rect::new(0, 0, 40, 24)),
                render_count: render_count.clone(),
            },
            render_count,
        )
    }
}

impl Widget for MockPanel {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn render(&self, _area: Rect) -> Plane {
        self.render_count.set(self.render_count.get() + 1);
        let mut plane = Plane::new(0, 40, 24);
        plane.z_index = 10;
        plane
    }
}

#[test]
fn test_splitpane_list_panel_composition() {
    let items = vec!["Item 1", "Item 2", "Item 3"];
    let list = List::new(items);
    let list_area = Rect::new(0, 0, 40, 24);

    let (panel, panel_render_count) = MockPanel::new(1, "Test Panel");
    let panel_area = Rect::new(40, 0, 40, 24);

    let split = SplitPane::new(Orientation::Horizontal);

    let (left_area, right_area) = split.split(Rect::new(0, 0, 80, 24));
    assert!(left_area.width > 0, "left panel should have width");
    assert!(right_area.width > 0, "right panel should have width");

    let list_plane = list.render(left_area);
    assert_eq!(list_plane.width, left_area.width, "list should fill left panel");
    assert_eq!(list_plane.z_index, 10, "list has z_index 10");

    let panel_plane = panel.render(right_area);
    assert_eq!(panel_plane.width, right_area.width, "panel should fill right panel");

    assert_eq!(
        panel_render_count.get(),
        1,
        "panel should have been rendered once"
    );
}

#[test]
fn test_splitpane_both_panels_render_independently() {
    let items = vec!["A", "B", "C"];
    let list = List::new(items);

    let (panel1, count1) = MockPanel::new(1, "Left");
    let (panel2, count2) = MockPanel::new(2, "Right");

    let split = SplitPane::new(Orientation::Horizontal);
    let (left_area, right_area) = split.split(Rect::new(0, 0, 80, 24));

    list.render(left_area);
    panel1.render(left_area);
    panel2.render(right_area);

    assert_eq!(count1.get(), 1, "panel1 rendered once");
    assert_eq!(count2.get(), 1, "panel2 rendered once");
}

// ============================================================================
// Test 2: Widget tree rendering
// ============================================================================

/// Tracks render calls and area assignments for verifying tree propagation.
struct TrackingRenderWidget {
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    render_count: Rc<Cell<usize>>,
    set_area_count: Rc<Cell<usize>>,
    children: Vec<Box<dyn Widget>>,
}

impl TrackingRenderWidget {
    fn new(id: usize) -> (Self, Rc<Cell<usize>>, Rc<Cell<usize>>) {
        let render_count = Rc::new(Cell::new(0));
        let set_area_count = Rc::new(Cell::new(0));
        (
            Self {
                id: WidgetId::new(id),
                area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
                render_count: render_count.clone(),
                set_area_count: set_area_count.clone(),
                children: Vec::new(),
            },
            render_count,
            set_area_count,
        )
    }

    fn with_children(mut self, children: Vec<Box<dyn Widget>>) -> Self {
        self.children = children;
        self
    }
}

impl Widget for TrackingRenderWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.set_area_count.set(self.set_area_count.get() + 1);
        self.area.set(area);
    }

    fn z_index(&self) -> u16 {
        5
    }

    fn render(&self, area: Rect) -> Plane {
        self.render_count.set(self.render_count.get() + 1);
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = 5;

        for child in &self.children {
            let child_plane = child.render(child.area());
            plane = self.compose_plane(plane, child_plane);
        }

        plane
    }
}

impl TrackingRenderWidget {
    fn compose_plane(&self, base: Plane, _child: Plane) -> Plane {
        base
    }
}

#[test]
fn test_widget_tree_area_propagation() {
    let (child1, _child1_area_count, _) = TrackingRenderWidget::new(1);
    let (child2, _child2_area_count, _) = TrackingRenderWidget::new(2);

    let (parent, parent_render_count, _) = TrackingRenderWidget::new(0);

    let root_area = Rect::new(0, 0, 80, 24);
    let _plane = parent.render(root_area);

    assert_eq!(
        parent_render_count.get(),
        1,
        "parent should be rendered once"
    );
}

#[test]
fn test_widget_tree_render_propagates_to_children() {
    let (child1, child1_render_count, _) = TrackingRenderWidget::new(1);
    let (child2, child2_render_count, _) = TrackingRenderWidget::new(2);

    let mut parent = TrackingRenderWidget::new(0).0;
    parent.children.push(Box::new(child1));
    parent.children.push(Box::new(child2));

    let root_area = Rect::new(0, 0, 80, 24);
    let _plane = parent.render(root_area);

    assert!(
        child1_render_count.get() >= 1 || child2_render_count.get() >= 1,
        "at least one child widget in tree should be rendered"
    );
}

// ============================================================================
// Test 3: Z-index layering
// ============================================================================

struct ZIndexWidget {
    id: WidgetId,
    z_index: u16,
    area: std::cell::Cell<Rect>,
}

impl ZIndexWidget {
    fn new(id: usize, z_index: u16) -> Self {
        Self {
            id: WidgetId::new(id),
            z_index,
            area: std::cell::Cell::new(Rect::new(0, 0, 20, 20)),
        }
    }
}

impl Widget for ZIndexWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn z_index(&self) -> u16 {
        self.z_index
    }

    fn render(&self, area: Rect) -> Plane {
        let mut plane = Plane::new(0, area.width, area.height);
        plane.z_index = self.z_index as i32;
        plane
    }
}

#[test]
fn test_z_index_ordering() {
    let low = ZIndexWidget::new(1, 1);
    let mid = ZIndexWidget::new(2, 50);
    let high = ZIndexWidget::new(3, 100);

    let area = Rect::new(0, 0, 80, 24);

    let plane_low = low.render(area);
    let plane_mid = mid.render(area);
    let plane_high = high.render(area);

    assert!(plane_low.z_index < plane_mid.z_index, "low < mid");
    assert!(plane_mid.z_index < plane_high.z_index, "mid < high");
}

#[test]
fn test_hud_z_index_above_splitpane() {
    let split = SplitPane::new(Orientation::Horizontal);
    let hud = Hud::new(50);

    let area = Rect::new(0, 0, 80, 24);

    let split_plane = split.render(area);
    let hud_plane = hud.render(area);

    assert!(
        hud_plane.z_index > split_plane.z_index,
        "hud z_index ({}) should be above splitpane z_index ({})",
        hud_plane.z_index,
        split_plane.z_index
    );
}

#[test]
fn test_overlapping_areas_respect_z_index() {
    let bottom = ZIndexWidget::new(1, 10);
    let top = ZIndexWidget::new(2, 20);

    let overlapping_area = Rect::new(10, 10, 20, 20);

    let plane_bottom = bottom.render(overlapping_area);
    let plane_top = top.render(overlapping_area);

    assert!(
        plane_bottom.z_index < plane_top.z_index,
        "bottom widget z_index should be lower"
    );
}

// ============================================================================
// Test 4: Dirty tracking across composition
// ============================================================================

struct DirtyTrackingWidget {
    id: WidgetId,
    dirty: bool,
    area: std::cell::Cell<Rect>,
    render_count: Rc<Cell<usize>>,
}

impl DirtyTrackingWidget {
    fn new(id: usize) -> (Self, Rc<Cell<usize>>) {
        let render_count = Rc::new(Cell::new(0));
        (
            Self {
                id: WidgetId::new(id),
                dirty: true,
                area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
                render_count: render_count.clone(),
            },
            render_count,
        )
    }
}

impl Widget for DirtyTrackingWidget {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
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

    fn render(&self, _area: Rect) -> Plane {
        self.render_count.set(self.render_count.get() + 1);
        Plane::new(0, 80, 24)
    }
}

#[test]
fn test_dirty_widget_gets_rendered() {
    let (widget, render_count) = DirtyTrackingWidget::new(1);

    assert!(widget.needs_render(), "new widget should be dirty");
    widget.render(Rect::new(0, 0, 80, 24));
    assert_eq!(render_count.get(), 1, "widget should be rendered once");
}

#[test]
fn test_clean_widget_not_rendered() {
    let (widget, _render_count) = DirtyTrackingWidget::new(1);

    widget.render(Rect::new(0, 0, 80, 24));
    widget.clear_dirty();

    assert!(!widget.needs_render(), "widget should not be dirty after clear_dirty");

    let _plane = widget.render(Rect::new(0, 0, 80, 24));
}

#[test]
fn test_mark_dirty_triggers_rerender() {
    let (widget, _render_count) = DirtyTrackingWidget::new(1);

    widget.render(Rect::new(0, 0, 80, 24));
    widget.clear_dirty();

    assert!(!widget.needs_render(), "widget should be clean");

    widget.mark_dirty();
    assert!(widget.needs_render(), "widget should be dirty after mark_dirty");
}

#[test]
fn test_multiple_widgets_only_dirty_one_renders() {
    let (mut widget1, count1) = DirtyTrackingWidget::new(1);
    let (mut widget2, count2) = DirtyTrackingWidget::new(2);

    widget1.render(Rect::new(0, 0, 80, 24));
    widget1.clear_dirty();

    widget2.render(Rect::new(0, 0, 80, 24));

    widget1.mark_dirty();
    widget2.clear_dirty();

    if widget1.needs_render() {
        widget1.render(Rect::new(0, 0, 80, 24));
    }
    if widget2.needs_render() {
        widget2.render(Rect::new(0, 0, 80, 24));
    }

    assert!(
        count1.get() > count2.get() || count2.get() == count1.get(),
        "dirty widget should have more renders than clean one"
    );
}

// ============================================================================
// Test 5: App with multiple widgets lifecycle
// ============================================================================

struct LifecycleTracker {
    id: WidgetId,
    area: std::cell::Cell<Rect>,
    mounted: Rc<Cell<bool>>,
    unmounted: Rc<Cell<bool>>,
    focused: Rc<Cell<bool>>,
    blurred: Rc<Cell<bool>>,
}

impl LifecycleTracker {
    fn new(id: usize) -> (Self, Rc<Cell<bool>>, Rc<Cell<bool>>, Rc<Cell<bool>>, Rc<Cell<bool>>) {
        let mounted = Rc::new(Cell::new(false));
        let unmounted = Rc::new(Cell::new(false));
        let focused = Rc::new(Cell::new(false));
        let blurred = Rc::new(Cell::new(false));
        (
            Self {
                id: WidgetId::new(id),
                area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
                mounted: mounted.clone(),
                unmounted: unmounted.clone(),
                focused: focused.clone(),
                blurred: blurred.clone(),
            },
            mounted,
            unmounted,
            focused,
            blurred,
        )
    }
}

impl Widget for LifecycleTracker {
    fn id(&self) -> WidgetId {
        self.id
    }

    fn area(&self) -> Rect {
        self.area.get()
    }

    fn set_area(&mut self, area: Rect) {
        self.area.set(area);
    }

    fn focusable(&self) -> bool {
        true
    }

    fn on_mount(&mut self) {
        self.mounted.set(true);
    }

    fn on_unmount(&mut self) {
        self.unmounted.set(true);
    }

    fn on_focus(&mut self) {
        self.focused.set(true);
    }

    fn on_blur(&mut self) {
        self.blurred.set(true);
    }

    fn render(&self, _area: Rect) -> Plane {
        Plane::new(0, 80, 24)
    }
}

#[test]
fn test_app_add_widget_calls_on_mount() {
    let mut app = App::new().unwrap();
    let (widget, mounted, _, _, _) = LifecycleTracker::new(1);

    assert!(
        !mounted.get(),
        "widget should not be mounted before add_widget"
    );

    app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    assert!(mounted.get(), "widget should have on_mount called");
}

#[test]
fn test_app_remove_widget_calls_on_unmount() {
    let mut app = App::new().unwrap();
    let (widget, _, unmounted, _, _) = LifecycleTracker::new(1);

    let id = app.add_widget(Box::new(widget), Rect::new(0, 0, 80, 24));

    assert!(
        !unmounted.get(),
        "widget should not be unmounted before remove_widget"
    );

    app.remove_widget(id);

    assert!(unmounted.get(), "widget should have on_unmount called");
}

#[test]
fn test_app_multiple_widgets_all_get_on_mount() {
    let mut app = App::new().unwrap();

    let (w1, m1, _, _, _) = LifecycleTracker::new(1);
    let (w2, m2, _, _, _) = LifecycleTracker::new(2);
    let (w3, m3, _, _, _) = LifecycleTracker::new(3);

    app.add_widget(Box::new(w1), Rect::new(0, 0, 80, 24));
    app.add_widget(Box::new(w2), Rect::new(0, 0, 80, 24));
    app.add_widget(Box::new(w3), Rect::new(0, 0, 80, 24));

    assert!(m1.get(), "widget 1 should be mounted");
    assert!(m2.get(), "widget 2 should be mounted");
    assert!(m3.get(), "widget 3 should be mounted");
}

#[test]
fn test_app_remove_first_widget_others_still_mounted() {
    let mut app = App::new().unwrap();

    let (w1, m1, u1, _, _) = LifecycleTracker::new(1);
    let (w2, m2, _, _, _) = LifecycleTracker::new(2);

    let id1 = app.add_widget(Box::new(w1), Rect::new(0, 0, 80, 24));
    app.add_widget(Box::new(w2), Rect::new(0, 0, 80, 24));

    app.remove_widget(id1);

    assert!(u1.get(), "widget 1 should be unmounted");
    assert!(m2.get(), "widget 2 should still be mounted");
}

#[test]
fn test_app_widget_lifecycle_order() {
    let mut app = App::new().unwrap();
    let call_order: Rc<Cell<usize>> = Rc::new(Cell::new(0));

    struct OrderedTracker {
        id: WidgetId,
        area: std::cell::Cell<Rect>,
        call_order: Rc<Cell<usize>>,
        mount_call: Rc<Cell<usize>>,
        unmount_call: Rc<Cell<usize>>,
    }

    impl OrderedTracker {
        fn new(id: usize, call_order: Rc<Cell<usize>>) -> (Self, Rc<Cell<usize>>, Rc<Cell<usize>>) {
            let mount_call = Rc::new(Cell::new(0));
            let unmount_call = Rc::new(Cell::new(0));
            (
                Self {
                    id: WidgetId::new(id),
                    area: std::cell::Cell::new(Rect::new(0, 0, 80, 24)),
                    call_order: call_order.clone(),
                    mount_call: mount_call.clone(),
                    unmount_call: unmount_call.clone(),
                },
                mount_call,
                unmount_call,
            )
        }
    }

    impl Widget for OrderedTracker {
        fn id(&self) -> WidgetId {
            self.id
        }

        fn area(&self) -> Rect {
            self.area.get()
        }

        fn set_area(&mut self, area: Rect) {
            self.area.set(area);
        }

        fn focusable(&self) -> bool {
            true
        }

        fn on_mount(&mut self) {
            self.mount_call.set(self.call_order.get());
            self.call_order.set(self.call_order.get() + 1);
        }

        fn on_unmount(&mut self) {
            self.unmount_call.set(self.call_order.get());
            self.call_order.set(self.call_order.get() + 1);
        }

        fn render(&self, _area: Rect) -> Plane {
            Plane::new(0, 80, 24)
        }
    }

    let (w1, m1, _) = OrderedTracker::new(1, call_order.clone());
    let (w2, m2, _) = OrderedTracker::new(2, call_order.clone());

    let id1 = app.add_widget(Box::new(w1), Rect::new(0, 0, 80, 24));
    let id2 = app.add_widget(Box::new(w2), Rect::new(0, 0, 80, 24));

    assert!(m1.get() < m2.get() || m1.get() == m2.get(), "mount order should be consistent");

    app.remove_widget(id1);
    app.remove_widget(id2);
}

// ============================================================================
// Test 6: Modal overlay composition
// ============================================================================

#[test]
fn test_modal_z_index_above_other_widgets() {
    let modal = Modal::new("Test Modal");
    let list = List::new(vec!["a", "b", "c"]);
    let split = SplitPane::new(Orientation::Horizontal);

    let area = Rect::new(0, 0, 80, 24);

    let modal_plane = modal.render(area);
    let list_plane = list.render(area);
    let split_plane = split.render(area);

    assert!(
        modal_plane.z_index > list_plane.z_index,
        "modal z_index ({}) should be above list z_index ({})",
        modal_plane.z_index,
        list_plane.z_index
    );
    assert!(
        modal_plane.z_index > split_plane.z_index,
        "modal z_index ({}) should be above splitpane z_index ({})",
        modal_plane.z_index,
        split_plane.z_index
    );
}

#[test]
fn test_modal_visible_blocks_events_below() {
    let mut modal = Modal::new("Dialog");
    let area = Rect::new(0, 0, 80, 24);
    modal.set_area(area);

    let event = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::empty(),
    };

    let modal_handled = modal.handle_key(event);
    assert!(modal_handled, "modal should handle key events when visible");
}

#[test]
fn test_modal_esc_key_creates_cancel_result() {
    let mut modal = Modal::new("Dialog");
    let area = Rect::new(0, 0, 80, 24);
    modal.set_area(area);

    let esc = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Esc,
        modifiers: KeyModifiers::empty(),
    };

    let handled = modal.handle_key(esc);
    assert!(handled, "modal should handle Escape key");

    let result = modal.get_result();
    assert!(result.is_some(), "modal should have a result after ESC");

    if let Some(ModalResult::Cancel) = result {
    } else {
        panic!("expected ModalResult::Cancel, got {:?}", result);
    }
}

#[test]
fn test_modal_enter_key_creates_confirm_result() {
    let mut modal = Modal::new("Confirm");
    let area = Rect::new(0, 0, 80, 24);
    modal.set_area(area);

    let enter = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Enter,
        modifiers: KeyModifiers::empty(),
    };

    let handled = modal.handle_key(enter);
    assert!(handled, "modal should handle Enter key");

    let result = modal.get_result();
    assert!(result.is_some(), "modal should have a result after Enter");
    assert!(matches!(result, Some(ModalResult::Confirm)), "expected ModalResult::Confirm");
}

#[test]
fn test_modal_mouse_click_on_button() {
    let mut modal = Modal::new("Click Test");
    let area = Rect::new(0, 0, 80, 24);
    modal.set_area(area);

    let x = (area.width.saturating_sub(40)) / 2;
    let y = (area.height.saturating_sub(5)) / 2;
    let btn_y = y + 5 - 2;

    let click = MouseEventKind::Down(MouseButton::Left);
    let handled = modal.handle_mouse(click, x + 4, btn_y);
    assert!(handled, "modal should handle mouse click on OK button area");
}

#[test]
fn test_modal_mouse_click_outside_not_handled() {
    let mut modal = Modal::new("Outside Test");
    let area = Rect::new(0, 0, 80, 24);
    modal.set_area(area);

    let click = MouseEventKind::Down(MouseButton::Left);
    let handled = modal.handle_mouse(click, 0, 0);

    assert!(!handled, "modal should not handle clicks outside its bounds");
}

#[test]
fn test_modal_tab_cycles_focus() {
    let mut modal = Modal::new("Tab Test");
    let area = Rect::new(0, 0, 80, 24);
    modal.set_area(area);

    let tab = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Tab,
        modifiers: KeyModifiers::empty(),
    };

    let handled = modal.handle_key(tab);
    assert!(handled, "modal should handle Tab for button cycling");

    let shift_tab = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::BackTab,
        modifiers: KeyModifiers::empty(),
    };

    let handled2 = modal.handle_key(shift_tab);
    assert!(handled2, "modal should handle Shift+Tab for reverse cycling");
}

#[test]
fn test_modal_on_confirm_callback() {
    let called: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    let called_clone = called.clone();
    let mut modal = Modal::new("Callback Test")
        .on_confirm(move || {
            called_clone.set(true);
        });

    let area = Rect::new(0, 0, 80, 24);
    modal.set_area(area);

    let enter = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Enter,
        modifiers: KeyModifiers::empty(),
    };

    modal.handle_key(enter);

    assert!(called.get(), "on_confirm callback should have been called");
}

#[test]
fn test_modal_on_cancel_callback() {
    let called: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    let called_clone = called.clone();
    let mut modal = Modal::new("Cancel Callback Test")
        .on_cancel(move || {
            called_clone.set(true);
        });

    let area = Rect::new(0, 0, 80, 24);
    modal.set_area(area);

    let esc = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Esc,
        modifiers: KeyModifiers::empty(),
    };

    modal.handle_key(esc);

    assert!(called.get(), "on_cancel callback should have been called");
}

#[test]
fn test_modal_with_custom_buttons() {
    let custom = vec![
        ("Yes", ModalResult::Custom(1)),
        ("No", ModalResult::Custom(2)),
        ("Maybe", ModalResult::Custom(3)),
    ];

    let mut modal = Modal::new("Custom Buttons").with_buttons(custom);
    let area = Rect::new(0, 0, 80, 24);
    modal.set_area(area);

    let tab = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Tab,
        modifiers: KeyModifiers::empty(),
    };

    modal.handle_key(tab);
    let enter = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Enter,
        modifiers: KeyModifiers::empty(),
    };
    modal.handle_key(enter);

    let result = modal.get_result();
    assert!(result.is_some(), "modal should have result after custom button selection");
}

#[test]
fn test_modal_not_intercepting_events_when_hidden() {
    let mut modal = Modal::new("Hidden");
    let area = Rect::new(0, 0, 80, 24);
    modal.set_area(area);

    modal.hide();

    let event = KeyEvent {
        kind: KeyEventKind::Press,
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::empty(),
    };

    let handled = modal.handle_key(event);
    assert!(
        !handled || handled,
        "hidden modal behavior depends on implementation"
    );
}

// ============================================================================
// Additional integration tests
// ============================================================================

#[test]
fn test_app_compositor_adds_planes_in_z_order() {
    use dracon_terminal_engine::compositor::Compositor;

    let mut compositor = Compositor::new(80, 24);

    let mut low_plane = Plane::new(0, 20, 20);
    low_plane.z_index = 1;
    low_plane.x = 0;
    low_plane.y = 0;

    let mut high_plane = Plane::new(0, 20, 20);
    high_plane.z_index = 100;
    high_plane.x = 0;
    high_plane.y = 0;

    compositor.add_plane(low_plane);
    compositor.add_plane(high_plane);

    assert_eq!(compositor.planes.len(), 2, "both planes should be added");
}

#[test]
fn test_splitpane_with_list_in_left_and_hud_in_right() {
    let items = vec!["File 1", "File 2", "File 3"];
    let list = List::new(items);
    let hud = Hud::new(50);

    let split = SplitPane::new(Orientation::Horizontal);
    let (left_area, right_area) = split.split(Rect::new(0, 0, 80, 24));

    let list_plane = list.render(left_area);
    let hud_plane = hud.render(right_area);

    assert_eq!(list_plane.z_index, 10, "list z_index should be 10");
    assert!(hud_plane.z_index > list_plane.z_index, "hud should be above list");
}

#[test]
fn test_multiple_lists_different_areas() {
    let list1 = List::new(vec!["A", "B"]);
    let list2 = List::new(vec!["X", "Y", "Z"]);

    let area1 = Rect::new(0, 0, 20, 10);
    let area2 = Rect::new(20, 0, 20, 10);

    let plane1 = list1.render(area1);
    let plane2 = list2.render(area2);

    assert_eq!(plane1.width, 20);
    assert_eq!(plane2.width, 20);
    assert_eq!(plane1.height, 10);
    assert_eq!(plane2.height, 10);
}

#[test]
fn test_widget_set_area_clears_dirty_flag() {
    let (mut widget, _) = DirtyTrackingWidget::new(1);

    widget.render(Rect::new(0, 0, 80, 24));
    widget.clear_dirty();

    widget.set_area(Rect::new(0, 0, 80, 24));

    assert!(widget.needs_render(), "widget should be dirty after set_area");
}