//! Tests for the ProgressBar widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::ProgressBar;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_progressbar_new() {
    let pb = ProgressBar::new(WidgetId::new(1));
    assert_eq!(pb.progress(), 0.0);
}

#[test]
fn test_progressbar_new_with_id() {
    let pb = ProgressBar::new(WidgetId::new(42));
    assert_eq!(pb.id(), WidgetId::new(42));
}

#[test]
fn test_progressbar_with_theme() {
    let pb = ProgressBar::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = pb.render(Rect::new(0, 0, 50, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_progressbar_default_progress_zero() {
    let pb = ProgressBar::new(WidgetId::new(1));
    assert_eq!(pb.progress(), 0.0);
}

// ============================================================================
// Progress Tests (values are 0.0-1.0, not 0-100)
// ============================================================================

#[test]
fn test_progressbar_set_progress() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_progress(0.5);
    assert_eq!(pb.progress(), 0.5);
}

#[test]
fn test_progressbar_set_progress_zero() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_progress(0.0);
    assert_eq!(pb.progress(), 0.0);
}

#[test]
fn test_progressbar_set_progress_full() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_progress(1.0);
    assert_eq!(pb.progress(), 1.0);
}

#[test]
fn test_progressbar_progress() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_progress(0.75);
    assert_eq!(pb.progress(), 0.75);
}

#[test]
fn test_progressbar_multiple_set_progress() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_progress(0.25);
    assert_eq!(pb.progress(), 0.25);
    pb.set_progress(0.5);
    assert_eq!(pb.progress(), 0.5);
    pb.set_progress(0.75);
    assert_eq!(pb.progress(), 0.75);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_progressbar_id() {
    let pb = ProgressBar::new(WidgetId::new(42));
    assert_eq!(pb.id(), WidgetId::new(42));
}

#[test]
fn test_progressbar_set_id() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_id(WidgetId::new(99));
    assert_eq!(pb.id(), WidgetId::new(99));
}

#[test]
fn test_progressbar_area() {
    let pb = ProgressBar::new(WidgetId::new(1));
    let area = pb.area();
    assert!(area.width > 0);
}

#[test]
fn test_progressbar_set_area() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_area(Rect::new(0, 0, 100, 3));
    assert_eq!(pb.area(), Rect::new(0, 0, 100, 3));
}

#[test]
fn test_progressbar_needs_render() {
    let pb = ProgressBar::new(WidgetId::new(1));
    assert!(pb.needs_render());
}

#[test]
fn test_progressbar_mark_dirty() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.clear_dirty();
    assert!(!pb.needs_render());
    pb.mark_dirty();
    assert!(pb.needs_render());
}

#[test]
fn test_progressbar_clear_dirty() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.clear_dirty();
    assert!(!pb.needs_render());
}

#[test]
fn test_progressbar_clear_dirty_after_set_progress() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.clear_dirty();
    pb.set_progress(0.5);
    assert!(pb.needs_render());
}

#[test]
fn test_progressbar_default_dirty() {
    let pb = ProgressBar::new(WidgetId::new(1));
    assert!(pb.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_progressbar_render_basic() {
    let pb = ProgressBar::new(WidgetId::new(1));
    let plane = pb.render(Rect::new(0, 0, 50, 1));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_progressbar_render_has_content() {
    let pb = ProgressBar::new(WidgetId::new(1));
    let plane = pb.render(Rect::new(0, 0, 50, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_progressbar_render_fills_bg() {
    let pb = ProgressBar::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = pb.render(Rect::new(0, 0, 50, 1));
    assert_eq!(plane.cells[0].bg, Theme::nord().bg);
}

#[test]
fn test_progressbar_render_wide() {
    let pb = ProgressBar::new(WidgetId::new(1));
    let plane = pb.render(Rect::new(0, 0, 100, 1));
    assert_eq!(plane.width, 100);
}

#[test]
fn test_progressbar_render_small() {
    let pb = ProgressBar::new(WidgetId::new(1));
    let plane = pb.render(Rect::new(0, 0, 5, 1));
    assert_eq!(plane.width, 5);
}

#[test]
fn test_progressbar_render_tall() {
    let pb = ProgressBar::new(WidgetId::new(1));
    let plane = pb.render(Rect::new(0, 0, 50, 3));
    assert_eq!(plane.height, 3);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_progressbar_theme_nord() {
    let pb = ProgressBar::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = pb.render(Rect::new(0, 0, 50, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_progressbar_theme_dracula() {
    let pb = ProgressBar::new(WidgetId::new(1)).with_theme(Theme::dracula());
    let plane = pb.render(Rect::new(0, 0, 50, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_progressbar_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let pb = ProgressBar::new(WidgetId::new(1)).with_theme(t);
        let _ = pb.render(Rect::new(0, 0, 50, 1));
    }
}

#[test]
fn test_progressbar_on_theme_change() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.on_theme_change(&Theme::nord());
    assert!(pb.needs_render());
}

#[test]
fn test_progressbar_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let pb = ProgressBar::new(WidgetId::new(1)).with_theme(t);
            let _ = pb.render(Rect::new(0, 0, 50, 1));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_progressbar_render_twice() {
    let pb = ProgressBar::new(WidgetId::new(1));
    let _ = pb.render(Rect::new(0, 0, 50, 1));
    let _ = pb.render(Rect::new(0, 0, 50, 1));
}

#[test]
fn test_progressbar_set_progress_and_render() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_progress(0.5);
    let plane = pb.render(Rect::new(0, 0, 50, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_progressbar_set_area_then_render() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_area(Rect::new(0, 0, 80, 2));
    let plane = pb.render(Rect::new(0, 0, 80, 2));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_progressbar_clamp_above_max() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_progress(1.5); // above 1.0
    assert_eq!(pb.progress(), 1.0); // should clamp to 1.0
}

#[test]
fn test_progressbar_clamp_below_min() {
    let mut pb = ProgressBar::new(WidgetId::new(1));
    pb.set_progress(-0.5); // below 0.0
    assert_eq!(pb.progress(), 0.0); // should clamp to 0.0
}
