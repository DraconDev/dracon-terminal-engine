//! Tests for the KeyValueGrid widget.

use std::collections::BTreeMap;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::KeyValueGrid;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_key_value_grid_new() {
    let kv = KeyValueGrid::new();
    let area = kv.area();
    assert!(area.width > 0);
}

#[test]
fn test_key_value_grid_new_with_id() {
    let kv = KeyValueGrid::with_id(WidgetId::new(42));
    assert_eq!(kv.id(), WidgetId::new(42));
}

#[test]
fn test_key_value_grid_with_theme() {
    let kv = KeyValueGrid::new().with_theme(Theme::nord());
    let plane = kv.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_key_value_grid_id() {
    let kv = KeyValueGrid::with_id(WidgetId::new(42));
    assert_eq!(kv.id(), WidgetId::new(42));
}

#[test]
fn test_key_value_grid_set_id() {
    let mut kv = KeyValueGrid::new();
    kv.set_id(WidgetId::new(99));
    assert_eq!(kv.id(), WidgetId::new(99));
}

#[test]
fn test_key_value_grid_area() {
    let kv = KeyValueGrid::new();
    let area = kv.area();
    assert!(area.width > 0);
}

#[test]
fn test_key_value_grid_set_area() {
    let mut kv = KeyValueGrid::new();
    kv.set_area(Rect::new(0, 0, 100, 30));
    assert_eq!(kv.area(), Rect::new(0, 0, 100, 30));
}

#[test]
fn test_key_value_grid_needs_render() {
    let kv = KeyValueGrid::new();
    assert!(kv.needs_render());
}

#[test]
fn test_key_value_grid_mark_dirty() {
    let mut kv = KeyValueGrid::new();
    kv.clear_dirty();
    assert!(!kv.needs_render());
    kv.mark_dirty();
    assert!(kv.needs_render());
}

#[test]
fn test_key_value_grid_clear_dirty() {
    let mut kv = KeyValueGrid::new();
    kv.clear_dirty();
    assert!(!kv.needs_render());
}

#[test]
fn test_key_value_grid_default_dirty() {
    let kv = KeyValueGrid::new();
    assert!(kv.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_key_value_grid_render_basic() {
    let kv = KeyValueGrid::new();
    let plane = kv.render(Rect::new(0, 0, 80, 20));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_key_value_grid_render_has_content() {
    let kv = KeyValueGrid::new();
    let plane = kv.render(Rect::new(0, 0, 80, 20));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_key_value_grid_render_wide() {
    let kv = KeyValueGrid::new();
    let plane = kv.render(Rect::new(0, 0, 120, 20));
    assert_eq!(plane.width, 120);
}

#[test]
fn test_key_value_grid_render_small() {
    let kv = KeyValueGrid::new();
    let plane = kv.render(Rect::new(0, 0, 30, 10));
    assert_eq!(plane.width, 30);
}

#[test]
fn test_key_value_grid_render_tall() {
    let kv = KeyValueGrid::new();
    let plane = kv.render(Rect::new(0, 0, 80, 50));
    assert_eq!(plane.height, 50);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_key_value_grid_theme_nord() {
    let kv = KeyValueGrid::new().with_theme(Theme::nord());
    let plane = kv.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_key_value_grid_theme_dracula() {
    let kv = KeyValueGrid::new().with_theme(Theme::dracula());
    let plane = kv.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_key_value_grid_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let kv = KeyValueGrid::new().with_theme(t);
        let _ = kv.render(Rect::new(0, 0, 80, 20));
    }
}

#[test]
fn test_key_value_grid_on_theme_change() {
    let mut kv = KeyValueGrid::new();
    kv.on_theme_change(&Theme::nord());
    assert!(kv.needs_render());
}

#[test]
fn test_key_value_grid_multiple_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark"];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let kv = KeyValueGrid::new().with_theme(t);
            let _ = kv.render(Rect::new(0, 0, 80, 20));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_key_value_grid_render_twice() {
    let kv = KeyValueGrid::new();
    let _ = kv.render(Rect::new(0, 0, 80, 20));
    let _ = kv.render(Rect::new(0, 0, 80, 20));
}

#[test]
fn test_key_value_grid_set_area_then_render() {
    let mut kv = KeyValueGrid::new();
    kv.set_area(Rect::new(0, 0, 100, 30));
    let plane = kv.render(Rect::new(0, 0, 100, 30));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_key_value_grid_set_pairs() {
    let mut kv = KeyValueGrid::new();
    let mut pairs = BTreeMap::new();
    pairs.insert("Key1".to_string(), "Value1".to_string());
    pairs.insert("Key2".to_string(), "Value2".to_string());
    kv.set_pairs(pairs);
    let plane = kv.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}
