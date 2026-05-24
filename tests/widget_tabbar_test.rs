//! Tests for the TabBar widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::tabbar::TabBar;
use ratatui::layout::Rect;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_tabbar_new() {
    let tabbar = TabBar::new(vec!["Home", "Settings", "Profile"]);
    assert_eq!(tabbar.active(), 0);
}

#[test]
fn test_tabbar_new_empty() {
    let tabbar = TabBar::new(Vec::<&str>::new());
    let plane = tabbar.render(Rect::new(0, 0, 80, 3));
    assert!(plane.width > 0);
}

#[test]
fn test_tabbar_new_single() {
    let tabbar = TabBar::new(vec!["Only One"]);
    assert_eq!(tabbar.active(), 0);
}

#[test]
fn test_tabbar_new_with_id() {
    let tabbar = TabBar::new_with_id(WidgetId::new(42), vec!["Tab1", "Tab2"]);
    assert_eq!(tabbar.id(), WidgetId::new(42));
}

#[test]
fn test_tabbar_with_theme() {
    let _ = TabBar::new(vec!["Home", "Settings"]).with_theme(Theme::nord());
}

// ============================================================================
// Tab State Tests
// ============================================================================

#[test]
fn test_tabbar_default_active_0() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3"]);
    assert_eq!(tabbar.active(), 0);
}

#[test]
fn test_tabbar_set_active_valid() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3"]);
    tabbar.set_active(1);
    assert_eq!(tabbar.active(), 1);
}

#[test]
fn test_tabbar_set_active_out_of_bounds() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    tabbar.set_active(100);
    assert!(tabbar.active() < 2);
}

#[test]
fn test_tabbar_set_active_single_tab() {
    let mut tabbar = TabBar::new(vec!["Only Tab"]);
    tabbar.set_active(0);
    assert_eq!(tabbar.active(), 0);
}

#[test]
fn test_tabbar_multiple_set_active() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3"]);
    tabbar.set_active(0);
    assert_eq!(tabbar.active(), 0);
    tabbar.set_active(1);
    assert_eq!(tabbar.active(), 1);
    tabbar.set_active(2);
    assert_eq!(tabbar.active(), 2);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_tabbar_id() {
    let _ = TabBar::new(vec!["Home", "Settings"]);
}

#[test]
fn test_tabbar_area() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    assert!(tabbar.area().width > 0);
}

#[test]
fn test_tabbar_set_area() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    tabbar.set_area(Rect::new(10, 20, 100, 5));
    assert_eq!(tabbar.area(), Rect::new(10, 20, 100, 5));
}

#[test]
fn test_tabbar_needs_render() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    assert!(tabbar.needs_render());
}

#[test]
fn test_tabbar_mark_dirty() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    tabbar.clear_dirty();
    assert!(!tabbar.needs_render());
    tabbar.mark_dirty();
    assert!(tabbar.needs_render());
}

#[test]
fn test_tabbar_clear_dirty() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    tabbar.clear_dirty();
    assert!(!tabbar.needs_render());
}

#[test]
fn test_tabbar_clear_dirty_then_mark() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    tabbar.clear_dirty();
    assert!(!tabbar.needs_render());
    tabbar.mark_dirty();
    assert!(tabbar.needs_render());
}

#[test]
fn test_tabbar_render() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3"]);
    assert_eq!(tabbar.render(Rect::new(0, 0, 80, 3)).width, 80);
}

#[test]
fn test_tabbar_z_index() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    assert_eq!(tabbar.z_index(), 10);
}

#[test]
fn test_tabbar_default_dirty_true() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    assert!(tabbar.needs_render());
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_tabbar_theme_nord() {
    let _ = TabBar::new(vec!["Home", "Settings"]).with_theme(Theme::nord());
}

#[test]
fn test_tabbar_theme_dracula() {
    let _ = TabBar::new(vec!["Home", "Settings"]).with_theme(Theme::dracula());
}

#[test]
fn test_tabbar_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let _ = TabBar::new(vec!["Home", "Settings"]).with_theme(t);
    }
}

#[test]
fn test_tabbar_theme_solarized_dark() {
    if let Some(t) = Theme::from_name("solarized_dark") {
        let _ = TabBar::new(vec!["Home", "Settings"]).with_theme(t);
    }
}

#[test]
fn test_tabbar_on_theme_change() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    tabbar.on_theme_change(&Theme::nord());
    assert!(tabbar.needs_render());
}

// ============================================================================
// Rendering Tests
// ============================================================================

#[test]
fn test_tabbar_render_fills_bg() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    let plane = tabbar.render(Rect::new(0, 0, 80, 3));
    assert_eq!(plane.cells[0].bg, Theme::default().bg);
}

#[test]
fn test_tabbar_render_has_content() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    let plane = tabbar.render(Rect::new(0, 0, 80, 3));
    assert!(plane.cells.iter().any(|c| c.char != '\0'));
}

#[test]
fn test_tabbar_render_minimal_area() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    assert_eq!(tabbar.render(Rect::new(0, 0, 5, 1)).width, 5);
}

#[test]
fn test_tabbar_render_wide_area() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3", "Tab4"]);
    assert_eq!(tabbar.render(Rect::new(0, 0, 200, 3)).width, 200);
}

#[test]
fn test_tabbar_render_small_area() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    assert_eq!(tabbar.render(Rect::new(0, 0, 1, 1)).width, 1);
}

#[test]
fn test_tabbar_render_medium_area() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3"]);
    let plane = tabbar.render(Rect::new(0, 0, 100, 3));
    assert_eq!(plane.width, 100);
    assert_eq!(plane.height, 3);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_tabbar_many_tabs() {
    let tabs: Vec<&str> = (0..50).map(|i| Box::leak(format!("Tab{}", i).into_boxed_str()) as &str).collect();
    let tabbar = TabBar::new(tabs);
    let _ = tabbar.render(Rect::new(0, 0, 200, 3));
}

#[test]
fn test_tabbar_unicode_tabs() {
    let _ = TabBar::new(vec!["日本語", "العربية", "🎉"]);
}

#[test]
fn test_tabbar_long_tabs() {
    let long = "A".repeat(100);
    let tabbar = TabBar::new(vec!["Tab1", &long]);
    let _ = tabbar.render(Rect::new(0, 0, 80, 3));
}

#[test]
fn test_tabbar_empty_string_tabs() {
    let _ = TabBar::new(vec!["", "", ""]);
}

#[test]
fn test_tabbar_active_clamps() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    tabbar.set_active(100);
    assert!(tabbar.active() < 2);
}

#[test]
fn test_tabbar_active_stays_valid() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3"]);
    tabbar.set_active(0);
    assert_eq!(tabbar.active(), 0);
    tabbar.set_active(5);
    assert!(tabbar.active() < 3);
}

// ============================================================================
// Additional Tests
// ============================================================================

#[test]
fn test_tabbar_set_active_same() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3"]);
    tabbar.set_active(1);
    assert_eq!(tabbar.active(), 1);
    tabbar.set_active(1);
    assert_eq!(tabbar.active(), 1);
}

#[test]
fn test_tabbar_render_twice() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    let _ = tabbar.render(Rect::new(0, 0, 80, 3));
    let _ = tabbar.render(Rect::new(0, 0, 80, 3));
}

#[test]
fn test_tabbar_set_area_then_render() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    tabbar.set_area(Rect::new(0, 0, 100, 5));
    let plane = tabbar.render(Rect::new(0, 0, 100, 5));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_tabbar_different_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark", "catppuccin_mocha"];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let _ = TabBar::new(vec!["Tab1", "Tab2"]).with_theme(t);
        }
    }
}
