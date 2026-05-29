//! Tests for the DebugOverlay widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::DebugOverlay;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_debug_overlay_new() {
    let d = DebugOverlay::new(WidgetId::new(1));
    let area = d.area();
    assert!(area.width > 0);
}

#[test]
fn test_debug_overlay_new_with_id() {
    let d = DebugOverlay::new(WidgetId::new(42));
    assert_eq!(d.id(), WidgetId::new(42));
}

#[test]
fn test_debug_overlay_with_theme() {
    let d = DebugOverlay::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

// ============================================================================
// Line Management Tests
// ============================================================================

#[test]
fn test_debug_overlay_add_line() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.add_line("Test line");
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_debug_overlay_set_lines() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.set_lines(vec![
        "Line1".to_string(),
        "Line2".to_string(),
        "Line3".to_string(),
    ]);
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_debug_overlay_clear() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.add_line("Test");
    d.clear();
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_debug_overlay_add_many_lines() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    for i in 0..20 {
        d.add_line(&format!("Line {}", i));
    }
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_debug_overlay_id() {
    let d = DebugOverlay::new(WidgetId::new(42));
    assert_eq!(d.id(), WidgetId::new(42));
}

#[test]
fn test_debug_overlay_set_id() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.set_id(WidgetId::new(99));
    assert_eq!(d.id(), WidgetId::new(99));
}

#[test]
fn test_debug_overlay_area() {
    let d = DebugOverlay::new(WidgetId::new(1));
    let area = d.area();
    assert!(area.width > 0);
}

#[test]
fn test_debug_overlay_set_area() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.set_area(Rect::new(0, 0, 100, 30));
    assert_eq!(d.area(), Rect::new(0, 0, 100, 30));
}

#[test]
fn test_debug_overlay_needs_render() {
    let d = DebugOverlay::new(WidgetId::new(1));
    assert!(d.needs_render());
}

#[test]
fn test_debug_overlay_mark_dirty() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.clear_dirty();
    assert!(!d.needs_render());
    d.mark_dirty();
    assert!(d.needs_render());
}

#[test]
fn test_debug_overlay_clear_dirty() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.clear_dirty();
    assert!(!d.needs_render());
}

#[test]
fn test_debug_overlay_default_dirty() {
    let d = DebugOverlay::new(WidgetId::new(1));
    assert!(d.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_debug_overlay_render_basic() {
    let d = DebugOverlay::new(WidgetId::new(1));
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_debug_overlay_render_has_content() {
    let d = DebugOverlay::new(WidgetId::new(1));
    let plane = d.render(Rect::new(0, 0, 80, 20));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_debug_overlay_render_wide() {
    let d = DebugOverlay::new(WidgetId::new(1));
    let plane = d.render(Rect::new(0, 0, 120, 20));
    assert_eq!(plane.width, 120);
}

#[test]
fn test_debug_overlay_render_small() {
    let d = DebugOverlay::new(WidgetId::new(1));
    let plane = d.render(Rect::new(0, 0, 20, 10));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_debug_overlay_render_tall() {
    let d = DebugOverlay::new(WidgetId::new(1));
    let plane = d.render(Rect::new(0, 0, 80, 50));
    assert_eq!(plane.height, 50);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_debug_overlay_theme_nord() {
    let d = DebugOverlay::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_debug_overlay_theme_dracula() {
    let d = DebugOverlay::new(WidgetId::new(1)).with_theme(Theme::dracula());
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_debug_overlay_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let d = DebugOverlay::new(WidgetId::new(1)).with_theme(t);
        let _ = d.render(Rect::new(0, 0, 80, 20));
    }
}

#[test]
fn test_debug_overlay_on_theme_change() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.on_theme_change(&Theme::nord());
    assert!(d.needs_render());
}

#[test]
fn test_debug_overlay_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let d = DebugOverlay::new(WidgetId::new(1)).with_theme(t);
            let _ = d.render(Rect::new(0, 0, 80, 20));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_debug_overlay_render_twice() {
    let d = DebugOverlay::new(WidgetId::new(1));
    let _ = d.render(Rect::new(0, 0, 80, 20));
    let _ = d.render(Rect::new(0, 0, 80, 20));
}

#[test]
fn test_debug_overlay_set_area_then_render() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.set_area(Rect::new(0, 0, 100, 30));
    let plane = d.render(Rect::new(0, 0, 100, 30));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_debug_overlay_add_empty_line() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.add_line("");
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_debug_overlay_unicode_lines() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.set_lines(vec!["ホーム".to_string(), "設定".to_string()]);
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_debug_overlay_long_lines() {
    let mut d = DebugOverlay::new(WidgetId::new(1));
    d.add_line(&"A".repeat(200));
    let plane = d.render(Rect::new(0, 0, 80, 20));
    assert!(plane.width > 0);
}
