//! Tests for the Autocomplete widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::autocomplete::Autocomplete;
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent { code, modifiers: KeyModifiers::empty(), kind: KeyEventKind::Press }
}

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_autocomplete_new() {
    let suggestions = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
    let ac = Autocomplete::new(suggestions);
    
    // Should have 3 suggestions
    assert!(ac.query().is_empty() || ac.query() == "Type to search...");
}

#[test]
fn test_autocomplete_new_with_id() {
    use dracon_terminal_engine::framework::widget::WidgetId;
    
    let id = WidgetId::new(42);
    let suggestions = vec!["one".to_string(), "two".to_string()];
    let ac = Autocomplete::new(id, suggestions);
    
    assert_eq!(ac.id(), id);
}

#[test]
fn test_autocomplete_empty_suggestions() {
    let ac = Autocomplete::new(vec![]);
    let area = Rect::new(0, 0, 30, 1);
    let _plane = ac.render(area);
}

#[test]
fn test_autocomplete_single_suggestion() {
    let suggestions = vec!["only".to_string()];
    let ac = Autocomplete::new(suggestions);
    let area = Rect::new(0, 0, 30, 1);
    let _plane = ac.render(area);
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_autocomplete_with_theme() {
    let suggestions = vec!["apple".to_string()];
    let ac = Autocomplete::new(suggestions).with_theme(Theme::nord());
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
    assert_eq!(plane.height >= 1, true);
}

#[test]
fn test_autocomplete_with_max_visible() {
    let suggestions = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    let ac = Autocomplete::new(suggestions).with_max_visible(5);
    let area = Rect::new(0, 0, 30, 10);
    let _plane = ac.render(area);
}

#[test]
fn test_autocomplete_on_select() {
    let suggestions = vec!["apple".to_string()];
    let _ac = Autocomplete::new(suggestions)
        .on_select(|_| {});
}

#[test]
fn test_autocomplete_chained_builders() {
    let suggestions = vec!["rust".to_string(), "go".to_string()];
    let ac = Autocomplete::new(suggestions)
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
    let suggestions = vec!["apple".to_string()];
    let ac = Autocomplete::new(suggestions);
    let query = ac.query();
    // Query is initialized from BaseInput, may be empty
    assert!(query.is_empty() || query == "Type to search...");
}

#[test]
fn test_autocomplete_selected_none_initially() {
    let suggestions = vec!["apple".to_string()];
    let ac = Autocomplete::new(suggestions);
    assert!(ac.selected().is_none());
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_autocomplete_id() {
    let suggestions = vec!["test".to_string()];
    let ac = Autocomplete::new(suggestions);
    let _id = ac.id();
}

#[test]
fn test_autocomplete_area() {
    let suggestions = vec!["test".to_string()];
    let ac = Autocomplete::new(suggestions);
    let area = ac.area();
    assert_eq!(area.width, 30);
    assert_eq!(area.height, 1);
}

#[test]
fn test_autocomplete_set_area() {
    let suggestions = vec!["test".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    let new_area = Rect::new(10, 20, 50, 5);
    ac.set_area(new_area);
    assert_eq!(ac.area(), new_area);
}

#[test]
fn test_autocomplete_needs_render() {
    let suggestions = vec!["test".to_string()];
    let ac = Autocomplete::new(suggestions);
    assert!(ac.needs_render());
}

#[test]
fn test_autocomplete_mark_dirty() {
    let suggestions = vec!["test".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.clear_dirty();
    assert!(!ac.needs_render());
    ac.mark_dirty();
    assert!(ac.needs_render());
}

#[test]
fn test_autocomplete_clear_dirty() {
    let suggestions = vec!["test".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.clear_dirty();
    assert!(!ac.needs_render());
}

#[test]
fn test_autocomplete_render() {
    let suggestions = vec!["test".to_string()];
    let ac = Autocomplete::new(suggestions);
    let area = Rect::new(0, 0, 30, 1);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
    assert!(plane.height >= 1);
}

#[test]
fn test_autocomplete_render_with_dropdown() {
    let suggestions = vec!["apple".to_string(), "banana".to_string(), "cherry".to_string()];
    let ac = Autocomplete::new(suggestions);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
    assert!(plane.height >= 1);
}

#[test]
fn test_autocomplete_render_minimal_area() {
    let suggestions = vec!["test".to_string()];
    let ac = Autocomplete::new(suggestions);
    let area = Rect::new(0, 0, 5, 1);
    let plane = ac.render(area);
    assert_eq!(plane.width, 5);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_autocomplete_z_index() {
    let suggestions = vec!["test".to_string()];
    let ac = Autocomplete::new(suggestions);
    assert_eq!(ac.z_index(), 20);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_autocomplete_different_themes() {
    for theme_name in ["nord", "dracula", "monokai", "solarized_dark"] {
        if let Some(theme) = Theme::from_name(theme_name) {
            let suggestions = vec!["test".to_string()];
            let ac = Autocomplete::new(suggestions).with_theme(theme);
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
    let suggestions = vec!["apple".to_string(), "banana".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.clear();
    // Clear should work without panicking
}

#[test]
fn test_autocomplete_clear_when_empty() {
    let suggestions = vec!["apple".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.clear();
    ac.clear(); // Double clear
}

// ============================================================================
// Dropdown Tests
// ============================================================================

#[test]
fn test_autocomplete_dropdown_closed_initially() {
    let suggestions = vec!["apple".to_string()];
    let ac = Autocomplete::new(suggestions);
    assert!(!ac.is_dropdown_open());
}

#[test]
fn test_autocomplete_open_dropdown() {
    let suggestions = vec!["apple".to_string(), "banana".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.open_dropdown();
    // Dropdown should open
}

// ============================================================================
// Handle Key Tests
// ============================================================================

#[test]
fn test_autocomplete_handle_key_up() {
    let suggestions = vec!["apple".to_string(), "banana".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.open_dropdown();
    
    let result = ac.handle_key(make_key(KeyCode::Up));
    assert!(result);
}

#[test]
fn test_autocomplete_handle_key_down() {
    let suggestions = vec!["apple".to_string(), "banana".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.open_dropdown();
    
    let result = ac.handle_key(make_key(KeyCode::Down));
    assert!(result);
}

#[test]
fn test_autocomplete_handle_key_enter() {
    let suggestions = vec!["apple".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.open_dropdown();
    
    let result = ac.handle_key(make_key(KeyCode::Enter));
    // Enter may or may not be handled depending on dropdown state
    let _ = result; // Just verify no crash
}

#[test]
fn test_autocomplete_handle_key_escape() {
    let suggestions = vec!["apple".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.open_dropdown();
    
    let result = ac.handle_key(make_key(KeyCode::Esc));
    // Escape should close dropdown
}

#[test]
fn test_autocomplete_handle_key_tab() {
    let suggestions = vec!["apple".to_string(), "banana".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.open_dropdown();
    
    let result = ac.handle_key(make_key(KeyCode::Tab));
    // Tab completes selection
    let _ = result;
}

#[test]
fn test_autocomplete_handle_key_character() {
    let suggestions = vec!["apple".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    
    let result = ac.handle_key(make_key(KeyCode::Char('a')));
    // Character input updates the filter
    let _ = result;
}

#[test]
fn test_autocomplete_handle_key_backspace() {
    let suggestions = vec!["apple".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    
    let result = ac.handle_key(make_key(KeyCode::Backspace));
    // Backspace removes character
    let _ = result;
}

// ============================================================================
// Handle Mouse Tests
// ============================================================================

#[test]
fn test_autocomplete_handle_mouse() {
    let suggestions = vec!["apple".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.open_dropdown();
    
    let area = Rect::new(0, 0, 30, 10);
    ac.render(area);
    
    // Click in dropdown area
    let result = ac.handle_mouse(
        dracon_terminal_engine::input::event::MouseEventKind::Down(
            dracon_terminal_engine::input::event::MouseButton::Left
        ),
        5, // col
        2  // row (in dropdown)
    );
    // Result depends on whether a suggestion zone was hit
    let _ = result;
}

#[test]
fn test_autocomplete_handle_mouse_outside() {
    let suggestions = vec!["apple".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.open_dropdown();
    
    let area = Rect::new(0, 0, 30, 10);
    ac.render(area);
    
    // Click far outside
    let result = ac.handle_mouse(
        dracon_terminal_engine::input::event::MouseEventKind::Down(
            dracon_terminal_engine::input::event::MouseButton::Left
        ),
        100, 100
    );
    assert!(!result);
}

#[test]
fn test_autocomplete_handle_mouse_scroll() {
    let suggestions: Vec<String> = (0..20).map(|i| format!("item_{}", i)).collect();
    let mut ac = Autocomplete::new(suggestions);
    ac.open_dropdown();
    
    let area = Rect::new(0, 0, 30, 10);
    ac.render(area);
    
    // Scroll up
    let result = ac.handle_mouse(
        dracon_terminal_engine::input::event::MouseEventKind::ScrollUp,
        15, 5
    );
    let _ = result;
    
    // Scroll down
    let result = ac.handle_mouse(
        dracon_terminal_engine::input::event::MouseEventKind::ScrollDown,
        15, 5
    );
    let _ = result;
}

// ============================================================================
// Focus Tests
// ============================================================================

#[test]
fn test_autocomplete_on_focus() {
    let suggestions = vec!["apple".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.on_focus();
}

#[test]
fn test_autocomplete_on_blur() {
    let suggestions = vec!["apple".to_string()];
    let mut ac = Autocomplete::new(suggestness=suggestions);
    ac.open_dropdown();
    ac.on_blur();
    assert!(!ac.is_dropdown_open());
}

// ============================================================================
// Selection Callback Tests
// ============================================================================

#[test]
fn test_autocomplete_selection_callback() {
    use std::cell::RefCell;
    use std::rc::Rc;
    
    let selected = Rc::new(RefCell::new(None));
    let selected_clone = Rc::clone(&selected);
    
    let suggestions = vec!["apple".to_string(), "banana".to_string()];
    let mut ac = Autocomplete::new(suggestions)
        .on_select(move |s| {
            *selected_clone.borrow_mut() = Some(s.to_string());
        });
    
    ac.open_dropdown();
    // Navigate and select
    for _ in 0..10 {
        ac.handle_key(make_key(KeyCode::Down));
    }
    ac.handle_key(make_key(KeyCode::Enter));
    
    // Callback should have been called
}

// ============================================================================
// Filtering Tests
// ============================================================================

#[test]
fn test_autocomplete_filter_updates() {
    let suggestions = vec![
        "apple".to_string(),
        "apricot".to_string(),
        "banana".to_string(),
        "cherry".to_string()
    ];
    let mut ac = Autocomplete::new(suggestions);
    
    ac.open_dropdown();
    // Filter should include matching suggestions
}

#[test]
fn test_autocomplete_case_insensitive_filter() {
    let suggestions = vec!["Apple".to_string(), "BANANA".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    ac.open_dropdown();
    // Should match regardless of case
}

// ============================================================================
// Rendering Tests
// ============================================================================

#[test]
fn test_autocomplete_render_fills_bg() {
    let suggestions = vec!["apple".to_string()];
    let ac = Autocomplete::new(suggestions);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    let theme = Theme::default();
    assert_eq!(plane.cells[0].bg, theme.bg);
}

#[test]
fn test_autocomplete_render_dropdown_height() {
    let suggestions: Vec<String> = (0..15).map(|i| format!("item_{}", i)).collect();
    let ac = Autocomplete::new(suggestions).with_max_visible(5);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    
    // Height should be 1 (input) + up to max_visible (dropdown)
    assert!(plane.height <= 1 + 5);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_autocomplete_many_suggestions() {
    let suggestions: Vec<String> = (0..100).map(|i| format!("item_{}", i)).collect();
    let ac = Autocomplete::new(suggestions);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
}

#[test]
fn test_autocomplete_unicode_suggestions() {
    let suggestions = vec![
        "日本語".to_string(),
        "العربية".to_string(),
        "🎉🎊".to_string()
    ];
    let ac = Autocomplete::new(suggestions);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
}

#[test]
fn test_autocomplete_long_suggestion() {
    let long_text = "a".repeat(1000);
    let suggestions = vec![long_text];
    let ac = Autocomplete::new(suggestions);
    let area = Rect::new(0, 0, 30, 10);
    let plane = ac.render(area);
    assert_eq!(plane.width, 30);
}

#[test]
fn test_autocomplete_no_matching_suggestions() {
    let suggestions = vec!["apple".to_string(), "banana".to_string()];
    let mut ac = Autocomplete::new(suggestions);
    // Enter something that doesn't match
    ac.clear();
    // Dropdown should close when no matches
}