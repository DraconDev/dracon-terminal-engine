//! Tests for the Spinner widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::Spinner;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_spinner_new() {
    let spinner = Spinner::new(WidgetId::new(1));
    let frame = spinner.current_frame();
    assert!(frame != '\0');
}

#[test]
fn test_spinner_new_with_id() {
    let spinner = Spinner::new(WidgetId::new(42));
    assert_eq!(spinner.id(), WidgetId::new(42));
}

#[test]
fn test_spinner_with_theme() {
    let spinner = Spinner::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = spinner.render(Rect::new(0, 0, 10, 1));
    assert!(plane.width > 0);
}

// ============================================================================
// Frames Tests
// ============================================================================
#[test]
fn test_spinner_with_frames() {
    let spinner = Spinner::new(WidgetId::new(1)).with_frames(vec!['|', '/', '-', '\\']);
    let frame = spinner.current_frame();
    assert!(frame == '|' || frame == '/' || frame == '-' || frame == '\\');
}

#[test]
fn test_spinner_with_single_frame() {
    let spinner = Spinner::new(WidgetId::new(1)).with_frames(vec!['*']);
    assert_eq!(spinner.current_frame(), '*');
}

#[test]
// ============================================================================
// Tick Tests
// ============================================================================
#[test]
fn test_spinner_tick() {
    let mut spinner = Spinner::new(WidgetId::new(1)).with_frames(vec!['1', '2', '3']);
    spinner.tick();
}

#[test]
fn test_spinner_tick_wraps() {
    let mut spinner = Spinner::new(WidgetId::new(1)).with_frames(vec!['1', '2']);
    let first = spinner.current_frame();
    spinner.tick();
    spinner.tick();
    let third = spinner.current_frame();
    // After 2 ticks with 2 frames, should be back to first
    assert_eq!(first, third);
}

#[test]
fn test_spinner_multiple_ticks() {
    let mut spinner = Spinner::new(WidgetId::new(1)).with_frames(vec!['a', 'b', 'c', 'd']);
    for _ in 0..10 {
        spinner.tick();
    }
    // Should still be valid
    let _ = spinner.current_frame();
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_spinner_id() {
    let spinner = Spinner::new(WidgetId::new(42));
    assert_eq!(spinner.id(), WidgetId::new(42));
}

#[test]
fn test_spinner_set_id() {
    let mut spinner = Spinner::new(WidgetId::new(1));
    spinner.set_id(WidgetId::new(99));
    assert_eq!(spinner.id(), WidgetId::new(99));
}

#[test]
fn test_spinner_area() {
    let spinner = Spinner::new(WidgetId::new(1));
    let area = spinner.area();
    assert!(area.width > 0);
}

#[test]
fn test_spinner_set_area() {
    let mut spinner = Spinner::new(WidgetId::new(1));
    spinner.set_area(Rect::new(0, 0, 20, 5));
    assert_eq!(spinner.area(), Rect::new(0, 0, 20, 5));
}

#[test]
fn test_spinner_needs_render() {
    let spinner = Spinner::new(WidgetId::new(1));
    assert!(spinner.needs_render());
}

#[test]
fn test_spinner_mark_dirty() {
    let mut spinner = Spinner::new(WidgetId::new(1));
    spinner.clear_dirty();
    assert!(!spinner.needs_render());
    spinner.mark_dirty();
    assert!(spinner.needs_render());
}

#[test]
fn test_spinner_clear_dirty() {
    let mut spinner = Spinner::new(WidgetId::new(1));
    spinner.clear_dirty();
    assert!(!spinner.needs_render());
}

#[test]
#[test]
fn test_spinner_default_dirty() {
    let spinner = Spinner::new(WidgetId::new(1));
    assert!(spinner.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_spinner_render_basic() {
    let spinner = Spinner::new(WidgetId::new(1));
    let plane = spinner.render(Rect::new(0, 0, 10, 1));
    assert_eq!(plane.width, 10);
}

#[test]
fn test_spinner_render_has_content() {
    let spinner = Spinner::new(WidgetId::new(1));
    let plane = spinner.render(Rect::new(0, 0, 10, 1));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_spinner_render_fills_bg() {
    let spinner = Spinner::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = spinner.render(Rect::new(0, 0, 10, 1));
    assert_eq!(plane.cells[0].bg, Theme::nord().bg);
}

#[test]
fn test_spinner_render_wide() {
    let spinner = Spinner::new(WidgetId::new(1));
    let plane = spinner.render(Rect::new(0, 0, 50, 1));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_spinner_render_small() {
    let spinner = Spinner::new(WidgetId::new(1));
    let plane = spinner.render(Rect::new(0, 0, 3, 1));
    assert_eq!(plane.width, 3);
}

#[test]
fn test_spinner_render_tall() {
    let spinner = Spinner::new(WidgetId::new(1));
    let plane = spinner.render(Rect::new(0, 0, 10, 5));
    assert_eq!(plane.height, 5);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_spinner_theme_nord() {
    let spinner = Spinner::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = spinner.render(Rect::new(0, 0, 10, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_spinner_theme_dracula() {
    let spinner = Spinner::new(WidgetId::new(1)).with_theme(Theme::dracula());
    let plane = spinner.render(Rect::new(0, 0, 10, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_spinner_theme_monokai() {
    if let Some(t) = Theme::from_name("monokai") {
        let spinner = Spinner::new(WidgetId::new(1)).with_theme(t);
        let _ = spinner.render(Rect::new(0, 0, 10, 1));
    }
}

#[test]
fn test_spinner_on_theme_change() {
    let mut spinner = Spinner::new(WidgetId::new(1));
    spinner.on_theme_change(&Theme::nord());
    assert!(spinner.needs_render());
}

#[test]
fn test_spinner_multiple_themes() {
    let themes = vec![
        "nord",
        "dracula",
        "monokai",
        "solarized_dark",
        "catppuccin_mocha",
    ];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let spinner = Spinner::new(WidgetId::new(1)).with_theme(t);
            let _ = spinner.render(Rect::new(0, 0, 10, 1));
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_spinner_render_twice() {
    let spinner = Spinner::new(WidgetId::new(1));
    let _ = spinner.render(Rect::new(0, 0, 10, 1));
    let _ = spinner.render(Rect::new(0, 0, 10, 1));
}

#[test]
fn test_spinner_tick_and_render() {
    let mut spinner = Spinner::new(WidgetId::new(1));
    spinner.tick();
    let plane = spinner.render(Rect::new(0, 0, 10, 1));
    assert!(plane.width > 0);
}

#[test]
fn test_spinner_set_area_then_render() {
    let mut spinner = Spinner::new(WidgetId::new(1));
    spinner.set_area(Rect::new(0, 0, 15, 3));
    let plane = spinner.render(Rect::new(0, 0, 15, 3));
    assert_eq!(plane.width, 15);
}

#[test]
fn test_spinner_current_frame() {
    let spinner = Spinner::new(WidgetId::new(1));
    let frame = spinner.current_frame();
    assert!(frame != '\0');
}
