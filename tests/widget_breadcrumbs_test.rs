//! Tests for the Breadcrumbs widget.

use std::path::Path;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::Breadcrumbs;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_breadcrumbs_new() {
    let crumbs = Breadcrumbs::new(vec!["Home".to_string(), "Settings".to_string()]);
    let area = crumbs.area();
    assert!(area.width > 0);
}

#[test]
fn test_breadcrumbs_new_with_id() {
    let crumbs = Breadcrumbs::new_with_id(WidgetId::new(42), vec!["A".to_string(), "B".to_string()]);
    assert_eq!(crumbs.id(), WidgetId::new(42));
}

#[test]
fn test_breadcrumbs_new_with_theme() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]).with_theme(Theme::nord());
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_breadcrumbs_single_segment() {
    let crumbs = Breadcrumbs::new(vec!["Only One".to_string()]);
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

// ============================================================================
// Path Tests
// ============================================================================

#[test]
fn test_breadcrumbs_from_path() {
    let crumbs = Breadcrumbs::from_path(Path::new("/home/user/documents"));
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_breadcrumbs_from_path_empty() {
    let crumbs = Breadcrumbs::from_path(Path::new("/"));
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_breadcrumbs_id() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    let _ = crumbs.id();
}

#[test]
fn test_breadcrumbs_set_id() {
    let mut crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    crumbs.set_id(WidgetId::new(99));
    assert_eq!(crumbs.id(), WidgetId::new(99));
}

#[test]
fn test_breadcrumbs_area() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    let area = crumbs.area();
    assert!(area.width > 0);
}

#[test]
fn test_breadcrumbs_set_area() {
    let mut crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    crumbs.set_area(Rect::new(0, 0, 100, 2));
    assert_eq!(crumbs.area(), Rect::new(0, 0, 100, 2));
}

#[test]
fn test_breadcrumbs_needs_render() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    assert!(crumbs.needs_render());
}

#[test]
fn test_breadcrumbs_mark_dirty() {
    let mut crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    crumbs.clear_dirty();
    assert!(!crumbs.needs_render());
    crumbs.mark_dirty();
    assert!(crumbs.needs_render());
}

#[test]
fn test_breadcrumbs_clear_dirty() {
    let mut crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    crumbs.clear_dirty();
    assert!(!crumbs.needs_render());
}

#[test]
fn test_breadcrumbs_default_dirty() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    assert!(crumbs.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_breadcrumbs_render_basic() {
    let crumbs = Breadcrumbs::new(vec!["Home".to_string(), "Settings".to_string()]);
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_breadcrumbs_render_has_content() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_breadcrumbs_render_wide() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    let plane = crumbs.render(Rect::new(0, 0, 120, 1));
    assert_eq!(plane.width, 120);
}

#[test]
fn test_breadcrumbs_render_small() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    let plane = crumbs.render(Rect::new(0, 0, 20, 1));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_breadcrumbs_render_tall() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    let plane = crumbs.render(Rect::new(0, 0, 80, 2));
    assert_eq!(plane.height, 2);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_breadcrumbs_theme_nord() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]).with_theme(Theme::nord());
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_breadcrumbs_theme_dracula() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]).with_theme(Theme::dracula());
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_breadcrumbs_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let crumbs = Breadcrumbs::new(vec!["Test".to_string()]).with_theme(t);
        let _ = crumbs.render(Rect::new(0, 0, 80, 1));
    }
}

#[test]
fn test_breadcrumbs_on_theme_change() {
    let mut crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    crumbs.on_theme_change(&Theme::nord());
    assert!(crumbs.needs_render());
}

#[test]
fn test_breadcrumbs_multiple_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark", "catppuccin_mocha"];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let crumbs = Breadcrumbs::new(vec!["Test".to_string()]).with_theme(t);
            let _ = crumbs.render(Rect::new(0, 0, 80, 1));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_breadcrumbs_render_twice() {
    let crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    let _ = crumbs.render(Rect::new(0, 0, 80, 1));
    let _ = crumbs.render(Rect::new(0, 0, 80, 1));
}

#[test]
fn test_breadcrumbs_set_area_then_render() {
    let mut crumbs = Breadcrumbs::new(vec!["Test".to_string()]);
    crumbs.set_area(Rect::new(0, 0, 100, 2));
    let plane = crumbs.render(Rect::new(0, 0, 100, 2));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_breadcrumbs_many_segments() {
    let segments: Vec<String> = (0..20).map(|i| format!("Segment{}", i)).collect();
    let crumbs = Breadcrumbs::new(segments);
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_breadcrumbs_unicode_segments() {
    let crumbs = Breadcrumbs::new(vec!["ホーム".to_string(), "設定".to_string()]);
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_breadcrumbs_empty_segment() {
    let crumbs = Breadcrumbs::new(vec!["".to_string()]);
    let plane = crumbs.render(Rect::new(0, 0, 80, 1));
    assert!(plane.width > 0);
}
