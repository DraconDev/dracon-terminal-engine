//! Integration tests for widget event handling (handle_key, handle_mouse).

use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::split::Orientation;
use dracon_terminal_engine::framework::widgets::{List, SearchInput, Slider};
use dracon_terminal_engine::input::event::{
    KeyCode, KeyEvent, KeyEventKind, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        kind: KeyEventKind::Press,
        code,
        modifiers: Default::default(),
    }
}

fn make_key_repeat(code: KeyCode) -> KeyEvent {
    KeyEvent {
        kind: KeyEventKind::Repeat,
        code,
        modifiers: Default::default(),
    }
}

// ========== List Event Tests ==========

#[test]
fn test_list_handle_key_down() {
    let items = vec!["a", "b", "c", "d", "e"];
    let mut list = List::new(items);
    assert_eq!(list.selected_index(), 0);

    list.handle_key(make_key(KeyCode::Down));
    assert_eq!(list.selected_index(), 1);

    list.handle_key(make_key(KeyCode::Down));
    assert_eq!(list.selected_index(), 2);
}

#[test]
fn test_list_handle_key_up() {
    let items = vec!["a", "b", "c"];
    let mut list = List::new(items);
    list.handle_key(make_key(KeyCode::Down));
    list.handle_key(make_key(KeyCode::Down));
    assert_eq!(list.selected_index(), 2);

    list.handle_key(make_key(KeyCode::Up));
    assert_eq!(list.selected_index(), 1);
}

#[test]
fn test_list_handle_key_home() {
    let items = vec!["a", "b", "c"];
    let mut list = List::new(items);
    list.handle_key(make_key(KeyCode::Down));
    list.handle_key(make_key(KeyCode::Down));
    assert_eq!(list.selected_index(), 2);

    list.handle_key(make_key(KeyCode::Home));
    assert_eq!(list.selected_index(), 0);
}

#[test]
fn test_list_handle_key_end() {
    let items = vec!["a", "b", "c"];
    let mut list = List::new(items);
    list.handle_key(make_key(KeyCode::End));
    assert_eq!(list.selected_index(), 2);
}

#[test]
fn test_list_handle_key_repeat_suppressed() {
    let items = vec!["a", "b", "c"];
    let mut list = List::new(items);
    assert_eq!(list.selected_index(), 0);

    list.handle_key(make_key_repeat(KeyCode::Down));
    assert_eq!(list.selected_index(), 0);
}

#[test]
fn test_list_handle_mouse_scroll() {
    let items: Vec<String> = (0..20).map(|i| format!("item{}", i)).collect();
    let mut list = List::new(items);
    list.set_visible_count(5);

    list.handle_mouse(MouseEventKind::ScrollDown, 0, 0);
    assert_eq!(list.selected_index(), 0);

    list.handle_mouse(MouseEventKind::ScrollUp, 0, 0);
    assert_eq!(list.selected_index(), 0);
}

// ========== SearchInput Event Tests ==========

#[test]
fn test_search_input_handle_key_char() {
    let mut si = SearchInput::new(WidgetId::default_id());
    assert_eq!(si.query(), "");

    si.handle_key(make_key(KeyCode::Char('h')));
    si.handle_key(make_key(KeyCode::Char('i')));
    assert_eq!(si.query(), "hi");
}

#[test]
fn test_search_input_handle_key_backspace() {
    let mut si = SearchInput::new(WidgetId::default_id());
    si.handle_key(make_key(KeyCode::Char('h')));
    si.handle_key(make_key(KeyCode::Char('i')));
    assert_eq!(si.query(), "hi");

    si.handle_key(make_key(KeyCode::Backspace));
    assert_eq!(si.query(), "h");
}

#[test]
fn test_search_input_handle_key_enter() {
    use std::cell::RefCell;
    use std::rc::Rc;
    let mut si = SearchInput::new(WidgetId::default_id());
    let submitted = Rc::new(RefCell::new(String::new()));
    let submitted_clone = submitted.clone();
    si = si.on_submit(move |q| *submitted_clone.borrow_mut() = q.to_string());

    si.handle_key(make_key(KeyCode::Char('t')));
    si.handle_key(make_key(KeyCode::Char('e')));
    si.handle_key(make_key(KeyCode::Char('s')));
    si.handle_key(make_key(KeyCode::Char('t')));
    si.handle_key(make_key(KeyCode::Enter));
    assert_eq!(*submitted.borrow(), "test");
}

#[test]
fn test_search_input_handle_key_repeat_suppressed() {
    let mut si = SearchInput::new(WidgetId::default_id());
    si.handle_key(make_key(KeyCode::Char('a')));
    si.handle_key(make_key_repeat(KeyCode::Char('b')));
    assert_eq!(si.query(), "a");
}

#[test]
fn test_search_input_handle_key_home_end() {
    let mut si = SearchInput::new(WidgetId::default_id());
    si.handle_key(make_key(KeyCode::Char('a')));
    si.handle_key(make_key(KeyCode::Char('b')));
    si.handle_key(make_key(KeyCode::Char('c')));
    assert_eq!(si.query(), "abc");

    si.handle_key(make_key(KeyCode::Home));
    si.handle_key(make_key(KeyCode::End));
    assert_eq!(si.query(), "abc");
}

// ========== Slider Event Tests ==========

#[test]
fn test_slider_handle_mouse_in_bounds() {
    let mut slider = Slider::new(WidgetId::default_id()).with_range(0.0, 100.0);
    slider.set_area(Rect::new(0, 0, 40, 3));
    assert_eq!(slider.value(), 50.0);

    slider.handle_mouse(MouseEventKind::Down(MouseButton::Left), 2, 0);
    let v = slider.value();
    assert!(v < 50.0);
}

#[test]
fn test_slider_handle_mouse_out_of_bounds_returns_false() {
    let mut slider = Slider::new(WidgetId::default_id()).with_range(0.0, 100.0);
    slider.set_area(Rect::new(0, 0, 40, 3));
    let initial = slider.value();

    let consumed = slider.handle_mouse(MouseEventKind::Down(MouseButton::Left), 100, 0);
    assert!(!consumed);
    assert_eq!(slider.value(), initial);
}

#[test]
fn test_slider_handle_key_not_implemented() {
    let mut slider = Slider::new(WidgetId::default_id()).with_range(0.0, 100.0);
    assert_eq!(slider.value(), 50.0);

    let result = slider.handle_key(make_key(KeyCode::Left));
    assert!(!result);
    assert_eq!(slider.value(), 50.0);
}

// ========== Checkbox Event Tests ==========

#[test]
fn test_checkbox_handle_key_enter() {
    use dracon_terminal_engine::framework::widgets::Checkbox;
    let mut cb = Checkbox::new(WidgetId::default_id(), "Test");
    assert!(!cb.is_checked());

    cb.handle_key(make_key(KeyCode::Enter));
    assert!(cb.is_checked());

    cb.handle_key(make_key(KeyCode::Enter));
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_handle_key_repeat_suppressed() {
    use dracon_terminal_engine::framework::widgets::Checkbox;
    let mut cb = Checkbox::new(WidgetId::default_id(), "Test");
    assert!(!cb.is_checked());

    cb.handle_key(make_key_repeat(KeyCode::Enter));
    assert!(!cb.is_checked());
}

#[test]
fn test_checkbox_handle_mouse_click() {
    use dracon_terminal_engine::framework::widgets::Checkbox;
    let mut cb = Checkbox::new(WidgetId::default_id(), "Test");
    assert!(!cb.is_checked());

    cb.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(cb.is_checked());
}

// ========== SplitPane Event Tests ==========

#[test]
fn test_split_pane_z_index() {
    use dracon_terminal_engine::framework::widgets::SplitPane;
    let split = SplitPane::new(Orientation::Horizontal);
    assert_eq!(split.z_index(), 5);
}

#[test]
fn test_split_pane_mouse_returns_false_for_non_drag() {
    use dracon_terminal_engine::framework::widgets::SplitPane;
    let mut split = SplitPane::new(Orientation::Horizontal);
    split.set_area(Rect::new(0, 0, 80, 24));

    let result = split.handle_mouse(MouseEventKind::Down(MouseButton::Left), 40, 12);
    assert!(!result);
}

#[test]
fn test_split_pane_mouse_drag_updates_ratio() {
    use dracon_terminal_engine::framework::widgets::SplitPane;
    let mut split = SplitPane::new(Orientation::Horizontal);
    split.set_area(Rect::new(0, 0, 80, 24));
    let initial_ratio = split.get_ratio();

    split.handle_mouse(MouseEventKind::Drag(MouseButton::Left), 60, 12);
    let new_ratio = split.get_ratio();
    assert_ne!(initial_ratio, new_ratio);
}

// ========== Toggle Event Tests ==========

#[test]
fn test_toggle_handle_key_enter() {
    use dracon_terminal_engine::framework::widgets::Toggle;
    let mut t = Toggle::new(WidgetId::default_id(), "Enable");
    assert!(!t.is_on());

    t.handle_key(make_key(KeyCode::Enter));
    assert!(t.is_on());

    t.handle_key(make_key(KeyCode::Enter));
    assert!(!t.is_on());
}

#[test]
fn test_toggle_handle_mouse_click() {
    use dracon_terminal_engine::framework::widgets::Toggle;
    let mut t = Toggle::new(WidgetId::default_id(), "Enable");
    assert!(!t.is_on());

    t.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(t.is_on());
}

// ========== Radio Event Tests ==========

#[test]
fn test_radio_handle_key_enter() {
    use dracon_terminal_engine::framework::widgets::Radio;
    let mut r = Radio::new(WidgetId::default_id(), "Option A");
    assert!(!r.is_selected());

    r.handle_key(make_key(KeyCode::Enter));
    assert!(r.is_selected());
}

#[test]
fn test_radio_handle_mouse_click() {
    use dracon_terminal_engine::framework::widgets::Radio;
    let mut r = Radio::new(WidgetId::default_id(), "Option A");
    assert!(!r.is_selected());

    r.handle_mouse(MouseEventKind::Down(MouseButton::Left), 0, 0);
    assert!(r.is_selected());
}

// ========== Widget Trait Tests ==========

#[test]
fn test_widget_id_default() {
    let id = WidgetId::default_id();
    assert_eq!(id, WidgetId::default_id());
}

#[test]
fn test_widget_id_equality() {
    let id1 = WidgetId::new(1);
    let id2 = WidgetId::new(1);
    let id3 = WidgetId::new(2);
    assert_eq!(id1, id2);
    assert_ne!(id1, id3);
}

#[test]
fn test_widget_id_uniqueness() {
    let ids: Vec<_> = (0..100).map(WidgetId::new).collect();
    for (i, id) in ids.iter().enumerate() {
        assert_eq!(id.0, i);
    }
}

#[test]
fn test_list_z_index() {
    let items = vec!["a", "b", "c"];
    let list = List::new(items);
    assert_eq!(list.z_index(), 10);
}

#[test]
fn test_list_area_set_and_get() {
    let items = vec!["a", "b", "c"];
    let mut list = List::new(items);
    let area = Rect::new(5, 10, 30, 15);
    list.set_area(area);
    assert_eq!(list.area(), area);
}
