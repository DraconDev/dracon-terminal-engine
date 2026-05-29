//! Tests for the SearchInput widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::SearchInput;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_search_input_new() {
    let si = SearchInput::new(WidgetId::new(1));
    assert_eq!(si.query(), "");
}

#[test]
fn test_search_input_new_with_id() {
    let si = SearchInput::new(WidgetId::new(42));
    assert_eq!(si.id(), WidgetId::new(42));
}

#[test]
fn test_search_input_with_theme() {
    let si = SearchInput::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = si.render(Rect::new(0, 0, 50, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_search_input_default_query_empty() {
    let si = SearchInput::new(WidgetId::new(1));
    assert_eq!(si.query(), "");
}

// ============================================================================
// Query Tests
// ============================================================================

#[test]
fn test_search_input_query() {
    let si = SearchInput::new(WidgetId::new(1));
    let q = si.query();
    assert_eq!(q, "");
}

#[test]
fn test_search_input_clear() {
    let mut si = SearchInput::new(WidgetId::new(1));
    si.clear();
    assert_eq!(si.query(), "");
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_search_input_id() {
    let si = SearchInput::new(WidgetId::new(42));
    assert_eq!(si.id(), WidgetId::new(42));
}

#[test]
fn test_search_input_set_id() {
    let mut si = SearchInput::new(WidgetId::new(1));
    si.set_id(WidgetId::new(99));
    assert_eq!(si.id(), WidgetId::new(99));
}

#[test]
fn test_search_input_area() {
    let si = SearchInput::new(WidgetId::new(1));
    let area = si.area();
    assert!(area.width > 0);
}

#[test]
fn test_search_input_set_area() {
    let mut si = SearchInput::new(WidgetId::new(1));
    si.set_area(Rect::new(0, 0, 100, 3));
    assert_eq!(si.area(), Rect::new(0, 0, 100, 3));
}

#[test]
fn test_search_input_needs_render() {
    let si = SearchInput::new(WidgetId::new(1));
    assert!(si.needs_render());
}

#[test]
fn test_search_input_mark_dirty() {
    let mut si = SearchInput::new(WidgetId::new(1));
    si.clear_dirty();
    assert!(!si.needs_render());
    si.mark_dirty();
    assert!(si.needs_render());
}

#[test]
fn test_search_input_clear_dirty() {
    let mut si = SearchInput::new(WidgetId::new(1));
    si.clear_dirty();
    assert!(!si.needs_render());
}

#[test]
fn test_search_input_default_dirty() {
    let si = SearchInput::new(WidgetId::new(1));
    assert!(si.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_search_input_render_basic() {
    let si = SearchInput::new(WidgetId::new(1));
    let plane = si.render(Rect::new(0, 0, 50, 1));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_search_input_render_has_content() {
    let si = SearchInput::new(WidgetId::new(1));
    let plane = si.render(Rect::new(0, 0, 50, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
#[test]
fn test_search_input_render_wide() {
    let si = SearchInput::new(WidgetId::new(1));
    let plane = si.render(Rect::new(0, 0, 100, 1));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_search_input_render_small() {
    let si = SearchInput::new(WidgetId::new(1));
    let plane = si.render(Rect::new(0, 0, 10, 1));
    assert_eq!(plane.width, 10);
}

#[test]
fn test_search_input_render_tall() {
    let si = SearchInput::new(WidgetId::new(1));
    let plane = si.render(Rect::new(0, 0, 50, 3));
    assert_eq!(plane.height, 3);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_search_input_theme_nord() {
    let si = SearchInput::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = si.render(Rect::new(0, 0, 50, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_search_input_theme_dracula() {
    let si = SearchInput::new(WidgetId::new(1)).with_theme(Theme::dracula());
    let plane = si.render(Rect::new(0, 0, 50, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_search_input_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let si = SearchInput::new(WidgetId::new(1)).with_theme(t);
        let _ = si.render(Rect::new(0, 0, 50, 1));
    }
}

#[test]
fn test_search_input_on_theme_change() {
    let mut si = SearchInput::new(WidgetId::new(1));
    si.on_theme_change(&Theme::nord());
    assert!(si.needs_render());
}

#[test]
fn test_search_input_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let si = SearchInput::new(WidgetId::new(1)).with_theme(t);
            let _ = si.render(Rect::new(0, 0, 50, 1));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_search_input_render_twice() {
    let si = SearchInput::new(WidgetId::new(1));
    let _ = si.render(Rect::new(0, 0, 50, 1));
    let _ = si.render(Rect::new(0, 0, 50, 1));
}

#[test]
fn test_search_input_clear_and_render() {
    let mut si = SearchInput::new(WidgetId::new(1));
    si.clear();
    let plane = si.render(Rect::new(0, 0, 50, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_search_input_set_area_then_render() {
    let mut si = SearchInput::new(WidgetId::new(1));
    si.set_area(Rect::new(0, 0, 80, 2));
    let plane = si.render(Rect::new(0, 0, 80, 2));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_search_input_empty_query() {
    let si = SearchInput::new(WidgetId::new(1));
    assert_eq!(si.query().len(), 0);
}

#[test]
fn test_search_input_clear_multiple_times() {
    let mut si = SearchInput::new(WidgetId::new(1));
    si.clear();
    si.clear();
    si.clear();
    assert_eq!(si.query(), "");
}
