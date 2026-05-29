//! Tests for the Hud widget.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::hud::Hud;

fn white() -> Color {
    Color::Rgb(255, 255, 255)
}
fn black() -> Color {
    Color::Rgb(0, 0, 0)
}
fn red() -> Color {
    Color::Rgb(255, 0, 0)
}
fn green() -> Color {
    Color::Rgb(0, 255, 0)
}
fn blue() -> Color {
    Color::Rgb(0, 0, 255)
}

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_hud_new() {
    let hud = Hud::new(100);
    assert!(hud.is_visible());
}

#[test]
fn test_hud_new_with_id() {
    use dracon_terminal_engine::framework::widget::WidgetId;
    let hud = Hud::new_with_id(WidgetId::new(42), 100);
    assert_eq!(hud.id(), WidgetId::new(42));
}

#[test]
fn test_hud_with_size() {
    let hud = Hud::new(100).with_size(50, 20);
    let (x, y) = hud.position();
    assert_eq!(x, 0);
    assert_eq!(y, 0);
}

#[test]
fn test_hud_with_theme() {
    let hud = Hud::new(100).with_theme(Theme::nord());
    let plane = hud.render_text(0, 0, "Test", white(), Color::Reset);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_default_visible() {
    let hud = Hud::new(100);
    assert!(hud.is_visible());
}

// ============================================================================
// Position Tests
// ============================================================================

#[test]
fn test_hud_position_always_zero() {
    let hud = Hud::new(100);
    let (x, y) = hud.position();
    assert_eq!(x, 0);
    assert_eq!(y, 0);
}

#[test]
fn test_hud_position_with_size() {
    let hud = Hud::new(50).with_size(100, 50);
    let (x, y) = hud.position();
    assert_eq!(x, 0);
    assert_eq!(y, 0);
}

// ============================================================================
// Visibility Tests
// ============================================================================

#[test]
fn test_hud_show() {
    let mut hud = Hud::new(100);
    hud.hide();
    assert!(!hud.is_visible());
    hud.show();
    assert!(hud.is_visible());
}

#[test]
fn test_hud_hide() {
    let mut hud = Hud::new(100);
    assert!(hud.is_visible());
    hud.hide();
    assert!(!hud.is_visible());
}

#[test]
fn test_hud_toggle_visibility() {
    let mut hud = Hud::new(100);
    assert!(hud.is_visible());
    hud.hide();
    assert!(!hud.is_visible());
    hud.show();
    assert!(hud.is_visible());
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_hud_id() {
    let hud = Hud::new(100);
    let _ = hud.id();
}

#[test]
fn test_hud_area() {
    let hud = Hud::new(100);
    let area = hud.area();
    assert!(area.width > 0);
    assert!(area.height > 0);
}

#[test]
fn test_hud_set_area() {
    use ratatui::layout::Rect;
    let mut hud = Hud::new(100);
    hud.set_area(Rect::new(0, 0, 50, 20));
    let area = hud.area();
    assert_eq!(area.width, 50);
}

#[test]
fn test_hud_needs_render() {
    let hud = Hud::new(100);
    assert!(hud.needs_render());
}

#[test]
fn test_hud_mark_dirty() {
    let mut hud = Hud::new(100);
    hud.clear_dirty();
    assert!(!hud.needs_render());
    hud.mark_dirty();
    assert!(hud.needs_render());
}

#[test]
fn test_hud_clear_dirty() {
    let mut hud = Hud::new(100);
    hud.clear_dirty();
    assert!(!hud.needs_render());
}

#[test]
fn test_hud_z_index() {
    let hud = Hud::new(200);
    assert_eq!(hud.z_index(), 200);
}

#[test]
fn test_hud_render() {
    let hud = Hud::new(100);
    let area = hud.area();
    let _plane = hud.render(area);
}

#[test]
fn test_hud_default_dirty() {
    let hud = Hud::new(100);
    assert!(hud.needs_render());
}

// ============================================================================
// Render Text Tests
// ============================================================================

#[test]
fn test_hud_render_text_basic() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_text(0, 0, "Hello", white(), Color::Reset);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_text_offset() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_text(5, 5, "Test", white(), Color::Reset);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_text_empty() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_text(0, 0, "", white(), Color::Reset);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_text_long() {
    let hud = Hud::new(100).with_size(30, 10);
    let long_text = "A".repeat(100);
    let plane = hud.render_text(0, 0, &long_text, white(), Color::Reset);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_text_unicode() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_text(0, 0, "日本語", white(), Color::Reset);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_text_different_colors() {
    let hud = Hud::new(100).with_size(30, 10);
    let _ = hud.render_text(0, 0, "Red", red(), Color::Reset);
    let _ = hud.render_text(0, 1, "Green", green(), Color::Reset);
    let _ = hud.render_text(0, 2, "Blue", blue(), Color::Reset);
}

#[test]
fn test_hud_render_text_background() {
    let hud = Hud::new(100).with_size(30, 10);
    let _ = hud.render_text(0, 0, "Test", white(), black());
    let _ = hud.render_text(0, 1, "Test", white(), blue());
}

#[test]
fn test_hud_render_text_boundary() {
    let hud = Hud::new(100).with_size(10, 5);
    let plane = hud.render_text(0, 0, "1234567890ABCDEF", white(), Color::Reset);
    assert!(plane.width > 0);
}

// ============================================================================
// Render Gauge Tests
// ============================================================================

#[test]
fn test_hud_render_gauge_basic() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_gauge(0, 0, "HP", 50.0, 100.0, 20);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_gauge_zero() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_gauge(0, 0, "HP", 0.0, 100.0, 20);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_gauge_full() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_gauge(0, 0, "HP", 100.0, 100.0, 20);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_gauge_partial() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_gauge(0, 0, "HP", 33.0, 100.0, 20);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_gauge_empty_label() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_gauge(0, 0, "", 50.0, 100.0, 20);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_gauge_multiple() {
    let hud = Hud::new(100).with_size(50, 20);
    let _ = hud.render_gauge(0, 0, "HP", 50.0, 100.0, 20);
    let _ = hud.render_gauge(0, 2, "MP", 75.0, 100.0, 20);
    let _ = hud.render_gauge(0, 4, "XP", 25.0, 100.0, 20);
}

#[test]
fn test_hud_render_gauge_over_max() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_gauge(0, 0, "HP", 150.0, 100.0, 20);
    assert!(plane.width > 0);
}

#[test]
fn test_hud_render_gauge_negative() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_gauge(0, 0, "HP", -10.0, 100.0, 20);
    assert!(plane.width > 0);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_hud_theme_nord() {
    let _ = Hud::new(100).with_theme(Theme::nord());
}

#[test]
fn test_hud_theme_dracula() {
    let _ = Hud::new(100).with_theme(Theme::dracula());
}

#[test]
fn test_hud_on_theme_change() {
    let mut hud = Hud::new(100);
    hud.on_theme_change(&Theme::nord());
    assert!(hud.needs_render());
}

#[test]
fn test_hud_multiple_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark"];
    for name in themes {
        if let Some(t) = Theme::from_name(name) {
            let _ = Hud::new(100).with_theme(t);
        }
    }
}

// ============================================================================
// Size Tests
// ============================================================================

#[test]
fn test_hud_size_various() {
    let sizes = vec![(10, 5), (20, 10), (50, 20), (100, 50)];
    for (w, h) in sizes {
        let hud = Hud::new(100).with_size(w, h);
        let plane = hud.render_text(0, 0, "Test", white(), Color::Reset);
        assert!(plane.width > 0);
    }
}

#[test]
fn test_hud_render_has_content() {
    let hud = Hud::new(100).with_size(30, 10);
    let plane = hud.render_text(0, 0, "Hello", white(), Color::Reset);
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_hud_render_text_at_boundaries() {
    let hud = Hud::new(100).with_size(30, 10);
    let _ = hud.render_text(0, 9, "Bottom", white(), Color::Reset);
    let _ = hud.render_text(29, 0, "Edge", white(), Color::Reset);
}

#[test]
fn test_hud_render_gauge_at_boundaries() {
    let hud = Hud::new(100).with_size(30, 10);
    let _ = hud.render_gauge(0, 9, "HP", 50.0, 100.0, 20);
    let _ = hud.render_gauge(10, 0, "MP", 50.0, 100.0, 20);
}

#[test]
fn test_hud_z_index_various() {
    let z_values = vec![0, 50, 100, 500, 1000];
    for z in z_values {
        let hud = Hud::new(z);
        assert_eq!(hud.z_index(), z);
    }
}
