//! Tests for the TabBar widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::tab_bar::TabBar;
use ratatui::layout::Rect;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_tabbar_new() {
    let tabs = vec!["Home", "Settings", "Profile"];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    assert_eq!(tabbar.active(), 0);
}

#[test]
fn test_tabbar_new_empty() {
    let tabs: Vec<&str> = vec![];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    assert!(tabbar.tabs().is_empty());
    let area = Rect::new(0, 0, 80, 3);
    let plane = tabbar.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_tabbar_new_single() {
    let tabs = vec!["Only One"];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    assert_eq!(tabbar.active(), 0);
}

#[test]
fn test_tabbar_new_with_id() {
    let id = WidgetId::new(42);
    let tabs = vec!["Tab1", "Tab2"];
    let tabbar = TabBar::new_with_id(id, tabs);
    assert_eq!(tabbar.id(), id);
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_tabbar_with_theme() {
    let tabs = vec!["Home", "Settings"];
    let tabbar = TabBar::new(tabs).with_theme(Theme::nord());
    let area = Rect::new(0, 0, 80, 3);
    let _plane = tabbar.render(area);
}

#[test]
fn test_tabbar_chained_builders() {
    let tabs = vec!["Home", "Profile", "Settings"];
    let tabbar = TabBar::new(tabs)
        .with_theme(Theme::cyberpunk());
    
    let area = Rect::new(0, 0, 80, 3);
    let _plane = tabbar.render(area);
}

// ============================================================================
// Tab Management Tests
// ============================================================================

#[test]
fn test_tabbar_default_active_0() {
    let tabs = vec!["Tab1", "Tab2", "Tab3"];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    assert_eq!(tabbar.active(), 0);
}

#[test]
fn test_tabbar_set_active_valid() {
    let tabs = vec!["Tab1", "Tab2", "Tab3"];
    let mut tabbar = TabBar::new(tabs);
    tabbar.set_active(1);
    assert_eq!(tabbar.active(), 1);
}

#[test]
fn test_tabbar_set_active_out_of_bounds() {
    let tabs = vec!["Tab1", "Tab2"];
    let mut tabbar = TabBar::new(tabs);
    tabbar.set_active(100);
    // Should stay at valid index or clamp
    assert!(tabbar.active() < tabs.len());
}

#[test]
fn test_tabbar_set_active_empty() {
    let tabs: Vec<&str> = vec![];
    let mut tabbar = TabBar::new(tabs);
    tabbar.set_active(0);
    // Should handle gracefully
}

#[test]
fn test_tabbar_default_tabs() {
    let tabs = vec!["Home", "Settings", "Profile"];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    assert_eq!(tabbar.tabs().len(), 3);
}

// ============================================================================
// Tab Methods Tests
// ============================================================================

#[test]
fn test_tabbar_tabs() {
    let tabs = vec!["Tab1", "Tab2", "Tab3"];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let retrieved = tabbar.tabs();
    assert_eq!(retrieved.len(), 3);
    assert_eq!(retrieved[0], "Tab1");
    assert_eq!(retrieved[1], "Tab2");
    assert_eq!(retrieved[2], "Tab3");
}

#[test]
fn test_tabbar_tabs_empty() {
    let tabs: Vec<&str> = vec![];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let retrieved = tabbar.tabs();
    assert_eq!(retrieved.len(), 0);
}

#[test]
fn test_tabbar_tab_count() {
    let tabs = vec!["Tab1", "Tab2", "Tab3", "Tab4"];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let count = tabbar.tab_count();
    assert_eq!(count, 4);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_tabbar_id() {
    let tabs = vec!["Home", "Settings"];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let _id = tabbar.id();
}

#[test]
fn test_tabbar_area() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    let area = tabbar.area();
    assert!(area.width > 0);
}

#[test]
fn test_tabbar_set_area() {
    let mut tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    let new_area = Rect::new(10, 20, 100, 5);
    tabbar.set_area(new_area);
    assert_eq!(tabbar.area(), new_area);
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
fn test_tabbar_render() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3"]);
    let area = Rect::new(0, 0, 80, 3);
    let plane = tabbar.render(area);
    assert_eq!(plane.width, 80);
}

#[test]
fn test_tabbar_z_index() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    assert_eq!(tabbar.z_index(), 10);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_tabbar_different_themes() {
    let tabs = vec!["Home", "Settings"];
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark"];
    
    for theme_name in themes {
        if let Some(theme) = Theme::from_name(theme_name) {
            let tabbar = TabBar::new(tabs.clone()).with_theme(theme);
            let area = Rect::new(0, 0, 80, 3);
            let plane = tabbar.render(area);
            assert!(plane.width > 0);
        }
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
    let area = Rect::new(0, 0, 80, 3);
    let plane = tabbar.render(area);
    let theme = Theme::default();
    assert_eq!(plane.cells[0].bg, theme.bg);
}

#[test]
fn test_tabbar_render_has_content() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    let area = Rect::new(0, 0, 80, 3);
    let plane = tabbar.render(area);
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content, "TabBar should render some content");
}

#[test]
fn test_tabbar_render_minimal_area() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    let area = Rect::new(0, 0, 5, 1);
    let plane = tabbar.render(area);
    assert_eq!(plane.width, 5);
}

#[test]
fn test_tabbar_render_wide_area() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3", "Tab4"]);
    let area = Rect::new(0, 0, 200, 3);
    let plane = tabbar.render(area);
    assert_eq!(plane.width, 200);
}

#[test]
fn test_tabbar_render_tall_height() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    let area = Rect::new(0, 0, 80, 10);
    let plane = tabbar.render(area);
    assert_eq!(plane.height, 10);
}

// ============================================================================
// Tab Count Tests
// ============================================================================

#[test]
fn test_tabbar_many_tabs() {
    let tabs: Vec<&str> = (0..50).map(|i| &format!("Tab{}", i)).collect();
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let area = Rect::new(0, 0, 200, 3);
    let _plane = tabbar.render(area);
}

#[test]
fn test_tabbar_unicode_tabs() {
    let tabs = vec!["日本語", "العربية", "🎉"];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let area = Rect::new(0, 0, 80, 3);
    let _plane = tabbar.render(area);
}

#[test]
fn test_tabbar_long_tabs() {
    let long_text = "A".repeat(100);
    let tabs = vec!["Tab1".to_string(), long_text];
    let tabbar = TabBar::new(vec!["Tab1", &long_text]);
    let area = Rect::new(0, 0, 80, 3);
    let _plane = tabbar.render(area);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_tabbar_empty_string_tabs() {
    let tabs = vec!["", "", ""];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let area = Rect::new(0, 0, 80, 3);
    let _plane = tabbar.render(area);
}

#[test]
fn test_tabbar_all_empty_tabs() {
    let tabs: Vec<&str> = vec!["", "", "", ""];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    let area = Rect::new(0, 0, 80, 3);
    let _plane = tabbar.render(area);
}

#[test]
fn test_tabbar_multiple_set_active() {
    let tabs = vec!["Tab1", "Tab2", "Tab3"];
    let mut tabbar = TabBar::new(tabs);
    
    tabbar.set_active(0);
    assert_eq!(tabbar.active(), 0);
    
    tabbar.set_active(1);
    assert_eq!(tabbar.active(), 1);
    
    tabbar.set_active(2);
    assert_eq!(tabbar.active(), 2);
}

#[test]
fn test_tabbar_multiple_themes() {
    let tabs = vec!["Home", "Settings"];
    let themes = vec![
        "nord", "dracula", "monokai", "solarized_dark", "catppuccin_mocha",
        "tokyo_night", "gruvbox_dark", "ayu_dark",
    ];
    
    for theme_name in themes {
        if let Some(theme) = Theme::from_name(theme_name) {
            let tabbar = TabBar::new(tabs.clone()).with_theme(theme);
            let area = Rect::new(0, 0, 80, 3);
            let _plane = tabbar.render(area);
        }
    }
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
fn test_tabbar_render_small_area() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    let area = Rect::new(0, 0, 1, 1);
    let plane = tabbar.render(area);
    assert_eq!(plane.width, 1);
}

#[test]
fn test_tabbar_active_clamps() {
    let tabs = vec!["Tab1", "Tab2"];
    let mut tabbar = TabBar::new(tabs);
    
    tabbar.set_active(100);
    assert!(tabbar.active() < tabs.len());
    
    tabbar.set_active(u8::MAX as usize);
    assert!(tabbar.active() < tabs.len());
}

#[test]
fn test_tabbar_default_dirty_true() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2"]);
    assert!(tabbar.needs_render());
}

#[test]
fn test_tabbar_tabs_consistency() {
    let tabs = vec!["A", "B", "C"];
    let tabbar = TabBar::new(tabs.iter().map(|s| s.as_str()).collect::<Vec<_>>());
    
    // Tabs should be consistent
    assert_eq!(tabbar.tab_count(), 3);
    let retrieved = tabbar.tabs();
    assert_eq!(retrieved.len(), 3);
}

#[test]
fn test_tabbar_render_medium_area() {
    let tabbar = TabBar::new(vec!["Tab1", "Tab2", "Tab3"]);
    let area = Rect::new(0, 0, 100, 3);
    let plane = tabbar.render(area);
    assert_eq!(plane.width, 100);
    assert_eq!(plane.height, 3);
}

#[test]
fn test_tabbar_active_stays_valid() {
    let tabs = vec!["Tab1", "Tab2", "Tab3"];
    let mut tabbar = TabBar::new(tabs);
    
    tabbar.set_active(0);
    assert_eq!(tabbar.active(), 0);
    
    tabbar.set_active(5); // Out of bounds
    assert!(tabbar.active() < tabs.len());
}