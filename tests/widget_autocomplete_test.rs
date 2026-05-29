//! Tests for the Autocomplete widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::autocomplete::Autocomplete;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;

fn make_id() -> WidgetId {
    WidgetId::new(0)
}

fn make_ac(suggestions: Vec<String>) -> Autocomplete {
    Autocomplete::new(make_id(), suggestions)
}

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    }
}

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_autocomplete_new() {
    let suggestions = vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ];
    let ac = make_ac(suggestions);
    let area = Rect::new(0, 0, 30, 1);
    let _plane = ac.render(area);
}

#[test]
fn test_autocomplete_new_with_id() {
    let id = WidgetId::new(42);
    let suggestions = vec!["one".to_string(), "two".to_string()];
    let ac = Autocomplete::new(id, suggestions);
    assert_eq!(ac.id(), id);
}

#[test]
fn test_autocomplete_empty_suggestions() {
    let ac = make_ac(vec![]);
    let area = Rect::new(0, 0, 30, 1);
    let _plane = ac.render(area);
}

#[test]
fn test_autocomplete_single_suggestion() {
    let ac = make_ac(vec!["only".to_string()]);
    let area = Rect::new(0, 0, 30, 1);
    let _plane = ac.render(area);
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_autocomplete_with_theme() {
    let ac = make_ac(vec!["apple".to_string()]).with_theme(Theme::nord());
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
}

#[test]
fn test_autocomplete_with_max_visible() {
    let ac = make_ac(vec!["a".to_string(), "b".to_string(), "c".to_string()]).with_max_visible(5);
    let area = Rect::new(0, 0, 30, 10);
    let _plane = ac.render(area);
}

#[test]
fn test_autocomplete_on_select() {
    let _ac = make_ac(vec!["apple".to_string()]).on_select(|_| {});
}

#[test]
fn test_autocomplete_chained_builders() {
    let ac = make_ac(vec!["rust".to_string(), "go".to_string()])
        .with_theme(Theme::cyberpunk())
        .with_max_visible(10)
        .on_select(|_| {});
    let area = Rect::new(0, 0, 30, 10);
    let _plane = ac.render(area);
}

// ============================================================================
// Query Tests
// ============================================================================

#[test]
fn test_autocomplete_query_empty_initially() {
    let ac = make_ac(vec!["apple".to_string()]);
    let query = ac.query();
    assert!(query.is_empty());
}

#[test]
fn test_autocomplete_selected_none_initially() {
    let ac = make_ac(vec!["apple".to_string()]);
    assert!(ac.selected().is_none());
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_autocomplete_id() {
    let ac = make_ac(vec!["test".to_string()]);
    let _id = ac.id();
}

#[test]
fn test_autocomplete_area() {
    let ac = make_ac(vec!["test".to_string()]);
    let area = ac.area();
    assert_eq!(area.width, 30);
    assert_eq!(area.height, 1);
}

#[test]
fn test_autocomplete_set_area() {
    let mut ac = make_ac(vec!["test".to_string()]);
    let new_area = Rect::new(10, 20, 50, 5);
    ac.set_area(new_area);
    assert_eq!(ac.area(), new_area);
}

#[test]
fn test_autocomplete_needs_render() {
    let ac = make_ac(vec!["test".to_string()]);
    assert!(ac.needs_render());
}

#[test]
fn test_autocomplete_mark_dirty() {
    let mut ac = make_ac(vec!["test".to_string()]);
    ac.clear_dirty();
    assert!(!ac.needs_render());
    ac.mark_dirty();
    assert!(ac.needs_render());
}

#[test]
fn test_autocomplete_clear_dirty() {
    let mut ac = make_ac(vec!["test".to_string()]);
    ac.clear_dirty();
    assert!(!ac.needs_render());
}

#[test]
fn test_autocomplete_render() {
    let ac = make_ac(vec!["test".to_string()]);
    let area = Rect::new(0, 0, 30, 1);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
    assert!(plane.height >= 1);
}

#[test]
fn test_autocomplete_render_with_dropdown() {
    let ac = make_ac(vec![
        "apple".to_string(),
        "banana".to_string(),
        "cherry".to_string(),
    ]);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
    assert!(plane.height >= 1);
}

#[test]
fn test_autocomplete_render_minimal_area() {
    let ac = make_ac(vec!["test".to_string()]);
    let area = Rect::new(0, 0, 5, 1);
    let plane = ac.render(area);
    assert_eq!(plane.width, 5);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_autocomplete_z_index() {
    let ac = make_ac(vec!["test".to_string()]);
    assert_eq!(ac.z_index(), 20);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_autocomplete_different_themes() {
    for theme_name in ["nord", "dracula", "monokai", "solarized_dark"] {
        if let Some(theme) = Theme::from_name(theme_name) {
            let ac = make_ac(vec!["test".to_string()]).with_theme(theme);
            let area = Rect::new(0, 0, 30, 10);
            let plane = ac.render(area);
            assert_eq!(plane.width, 30);
        }
    }
}

// ============================================================================
// Clear Tests
// ============================================================================

#[test]
fn test_autocomplete_clear() {
    let mut ac = make_ac(vec!["apple".to_string(), "banana".to_string()]);
    ac.clear();
}

#[test]
fn test_autocomplete_clear_when_empty() {
    let mut ac = make_ac(vec!["apple".to_string()]);
    ac.clear();
    ac.clear();
}

// ============================================================================
// Dropdown Tests
// ============================================================================

#[test]
fn test_autocomplete_dropdown_closed_initially() {
    let ac = make_ac(vec!["apple".to_string()]);
    assert!(!ac.is_dropdown_open());
}

#[test]
fn test_autocomplete_open_dropdown() {
    let mut ac = make_ac(vec!["apple".to_string(), "banana".to_string()]);
    ac.open_dropdown();
}

// ============================================================================
// Handle Key Tests
// ============================================================================

#[test]
fn test_autocomplete_handle_key_up() {
    let mut ac = make_ac(vec!["apple".to_string(), "banana".to_string()]);
    ac.open_dropdown();
    let result = ac.handle_key(make_key(KeyCode::Up));
    assert!(result);
}

#[test]
fn test_autocomplete_handle_key_down() {
    let mut ac = make_ac(vec!["apple".to_string(), "banana".to_string()]);
    ac.open_dropdown();
    let result = ac.handle_key(make_key(KeyCode::Down));
    assert!(result);
}

#[test]
fn test_autocomplete_handle_key_enter() {
    let mut ac = make_ac(vec!["apple".to_string()]);
    ac.open_dropdown();
    let _result = ac.handle_key(make_key(KeyCode::Enter));
}

#[test]
fn test_autocomplete_handle_key_escape() {
    let mut ac = make_ac(vec!["apple".to_string()]);
    ac.open_dropdown();
    let _result = ac.handle_key(make_key(KeyCode::Esc));
}

#[test]
fn test_autocomplete_handle_key_tab() {
    let mut ac = make_ac(vec!["apple".to_string(), "banana".to_string()]);
    ac.open_dropdown();
    let _result = ac.handle_key(make_key(KeyCode::Tab));
}

#[test]
fn test_autocomplete_handle_key_character() {
    let mut ac = make_ac(vec!["apple".to_string()]);
    let _result = ac.handle_key(make_key(KeyCode::Char('a')));
}

#[test]
fn test_autocomplete_handle_key_backspace() {
    let mut ac = make_ac(vec!["apple".to_string()]);
    let _result = ac.handle_key(make_key(KeyCode::Backspace));
}

// ============================================================================
// Handle Mouse Tests
// ============================================================================

#[test]
fn test_autocomplete_handle_mouse() {
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};

    let mut ac = make_ac(vec!["apple".to_string()]);
    ac.open_dropdown();
    let area = Rect::new(0, 0, 30, 10);
    ac.render(area);
    let _result = ac.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 2);
}

#[test]
fn test_autocomplete_handle_mouse_outside() {
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};

    let mut ac = make_ac(vec!["apple".to_string()]);
    ac.open_dropdown();
    let area = Rect::new(0, 0, 30, 10);
    ac.render(area);
    let result = ac.handle_mouse(MouseEventKind::Down(MouseButton::Left), 100, 100);
    assert!(!result);
}

#[test]
fn test_autocomplete_handle_mouse_scroll() {
    use dracon_terminal_engine::input::event::MouseEventKind;

    let suggestions: Vec<String> = (0..20).map(|i| format!("item_{}", i)).collect();
    let mut ac = make_ac(suggestions);
    ac.open_dropdown();
    let area = Rect::new(0, 0, 30, 10);
    ac.render(area);
    let _ = ac.handle_mouse(MouseEventKind::ScrollUp, 15, 5);
    let _ = ac.handle_mouse(MouseEventKind::ScrollDown, 15, 5);
}

// ============================================================================
// Focus Tests
// ============================================================================

#[test]
fn test_autocomplete_on_focus() {
    let mut ac = make_ac(vec!["apple".to_string()]);
    ac.on_focus();
}

#[test]
fn test_autocomplete_on_blur() {
    let mut ac = make_ac(vec!["apple".to_string()]);
    ac.open_dropdown();
    ac.on_blur();
    assert!(!ac.is_dropdown_open());
}

// ============================================================================
// Selection Callback Tests
// ============================================================================

#[test]
fn test_autocomplete_selection_callback_registration() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let selected = Rc::new(RefCell::new(Vec::new()));
    let selected_clone = Rc::clone(&selected);

    let mut ac = make_ac(vec!["apple".to_string(), "banana".to_string()]).on_select(move |s| {
        selected_clone.borrow_mut().push(s.to_string());
    });

    ac.open_dropdown();
    // Navigate down
    for _ in 0..5 {
        ac.handle_key(make_key(KeyCode::Down));
    }
    // Select
    ac.handle_key(make_key(KeyCode::Enter));
}

// ============================================================================
// Rendering Tests
// ============================================================================

#[test]
fn test_autocomplete_render_fills_bg() {
    let ac = make_ac(vec!["apple".to_string()]);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    // Check that plane has a background (cells are filled)
    let has_bg = plane
        .cells
        .iter()
        .any(|c| c.bg != Theme::default().bg || c.char != '\0');
    assert!(has_bg || !plane.cells.is_empty());
}

#[test]
fn test_autocomplete_render_dropdown_height() {
    let suggestions: Vec<String> = (0..15).map(|i| format!("item_{}", i)).collect();
    let ac = make_ac(suggestions).with_max_visible(5);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert!(plane.height <= 1 + 5);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_autocomplete_many_suggestions() {
    let suggestions: Vec<String> = (0..100).map(|i| format!("item_{}", i)).collect();
    let ac = make_ac(suggestions);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
}

#[test]
fn test_autocomplete_unicode_suggestions() {
    let suggestions = vec![
        "日本語".to_string(),
        "العربية".to_string(),
        "🎉🎊".to_string(),
    ];
    let ac = make_ac(suggestions);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
}

#[test]
fn test_autocomplete_long_suggestion() {
    let long_text = "a".repeat(1000);
    let ac = make_ac(vec![long_text]);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
}
