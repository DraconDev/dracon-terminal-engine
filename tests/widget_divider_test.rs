//! Tests for the Divider widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::divider::{
    Divider, DividerDirection, DividerStyle, LabelPosition,
};
use ratatui::layout::Rect;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_divider_new() {
    let divider = Divider::new();
    let area = Rect::new(0, 0, 80, 1);
    let plane = divider.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_divider_default() {
    let divider = Divider::default();
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_vertical() {
    let divider = Divider::vertical();
    let area = Rect::new(0, 0, 1, 30);
    let plane = divider.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_divider_with_label() {
    let divider = Divider::new().with_label("Section");
    let area = Rect::new(0, 0, 80, 1);
    let plane = divider.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_divider_with_theme() {
    let divider = Divider::new().with_theme(Theme::nord());
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_direction() {
    let divider = Divider::new().direction(DividerDirection::Vertical);
    let area = Rect::new(0, 0, 1, 30);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_style() {
    let divider = Divider::new().style(DividerStyle::Bold);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_label_position() {
    let divider = Divider::new()
        .with_label("Center")
        .label_position(LabelPosition::Center);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_chained_builders() {
    let divider = Divider::new()
        .with_theme(Theme::cyberpunk())
        .direction(DividerDirection::Horizontal)
        .style(DividerStyle::Double)
        .with_label("Test")
        .label_position(LabelPosition::Left);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

// ============================================================================
// Direction Tests
// ============================================================================

#[test]
fn test_divider_direction_horizontal() {
    let divider = Divider::new().direction(DividerDirection::Horizontal);
    let area = Rect::new(0, 0, 80, 1);
    let plane = divider.render(area);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_divider_direction_vertical() {
    let divider = Divider::vertical();
    let area = Rect::new(0, 0, 1, 30);
    let plane = divider.render(area);
    assert!(plane.height > 1);
}

#[test]
fn test_divider_default_is_horizontal() {
    let divider = Divider::new();
    let area = Rect::new(0, 0, 80, 1);
    let plane = divider.render(area);
    assert_eq!(plane.height, 1);
}

// ============================================================================
// Style Tests
// ============================================================================

#[test]
fn test_divider_style_solid() {
    let divider = Divider::new().style(DividerStyle::Solid);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_style_dashed() {
    let divider = Divider::new().style(DividerStyle::Dashed);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_style_double() {
    let divider = Divider::new().style(DividerStyle::Double);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_style_bold() {
    let divider = Divider::new().style(DividerStyle::Bold);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_all_styles() {
    let styles = vec![
        DividerStyle::Solid,
        DividerStyle::Dashed,
        DividerStyle::Double,
        DividerStyle::Bold,
    ];

    for style in styles {
        let divider = Divider::new().style(style);
        let area = Rect::new(0, 0, 80, 1);
        let plane = divider.render(area);
        assert!(plane.width > 0);
    }
}

// ============================================================================
// Label Position Tests
// ============================================================================

#[test]
fn test_divider_label_position_left() {
    let divider = Divider::new()
        .with_label("Left")
        .label_position(LabelPosition::Left);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_label_position_center() {
    let divider = Divider::new()
        .with_label("Center")
        .label_position(LabelPosition::Center);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_label_position_right() {
    let divider = Divider::new()
        .with_label("Right")
        .label_position(LabelPosition::Right);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_label_position_default() {
    let divider = Divider::new().with_label("Test");
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
    // Default should be Center
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_divider_id() {
    let divider = Divider::new();
    let _id = divider.id();
}

#[test]
fn test_divider_area() {
    let divider = Divider::new();
    let area = divider.area();
    assert!(area.width > 0);
}

#[test]
fn test_divider_set_area() {
    let mut divider = Divider::new();
    let new_area = Rect::new(10, 20, 100, 1);
    divider.set_area(new_area);
    assert_eq!(divider.area(), new_area);
}

#[test]
fn test_divider_needs_render() {
    let divider = Divider::new();
    assert!(divider.needs_render());
}

#[test]
fn test_divider_mark_dirty() {
    let mut divider = Divider::new();
    divider.clear_dirty();
    assert!(!divider.needs_render());
    divider.mark_dirty();
    assert!(divider.needs_render());
}

#[test]
fn test_divider_clear_dirty() {
    let mut divider = Divider::new();
    divider.clear_dirty();
    assert!(!divider.needs_render());
}

#[test]
fn test_divider_render() {
    let divider = Divider::new();
    let area = Rect::new(0, 0, 80, 1);
    let plane = divider.render(area);
    assert_eq!(plane.width, 80);
}

#[test]
fn test_divider_z_index() {
    let divider = Divider::new();
    assert_eq!(divider.z_index(), 5);
}

// ============================================================================
// Set Label Tests
// ============================================================================

#[test]
fn test_divider_set_label_some() {
    let mut divider = Divider::new();
    divider.set_label(Some("New Label"));
}

#[test]
fn test_divider_set_label_none() {
    let mut divider = Divider::new().with_label("Initial");
    divider.set_label(None);
}

#[test]
fn test_divider_set_label_empty() {
    let mut divider = Divider::new().with_label("Initial");
    divider.set_label(Some(""));
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_divider_different_themes() {
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark"];

    for theme_name in themes {
        if let Some(theme) = Theme::from_name(theme_name) {
            let divider = Divider::new().with_theme(theme);
            let area = Rect::new(0, 0, 80, 1);
            let plane = divider.render(area);
            assert!(plane.width > 0);
        }
    }
}

#[test]
fn test_divider_on_theme_change() {
    let mut divider = Divider::new();
    divider.on_theme_change(&Theme::nord());
    assert!(divider.needs_render());
}

// ============================================================================
// Rendering Tests
// ============================================================================

#[test]
fn test_divider_render_fills_bg() {
    let divider = Divider::new();
    let area = Rect::new(0, 0, 80, 1);
    let plane = divider.render(area);
    let theme = Theme::default();
    assert_eq!(plane.cells[0].bg, theme.bg);
}

#[test]
fn test_divider_render_has_content() {
    let divider = Divider::new();
    let area = Rect::new(0, 0, 80, 1);
    let plane = divider.render(area);
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content, "Divider should render some content");
}

#[test]
fn test_divider_render_minimal_area() {
    let divider = Divider::new();
    let area = Rect::new(0, 0, 5, 1);
    let plane = divider.render(area);
    assert_eq!(plane.width, 5);
}

#[test]
fn test_divider_render_wide_area() {
    let divider = Divider::new();
    let area = Rect::new(0, 0, 200, 1);
    let plane = divider.render(area);
    assert_eq!(plane.width, 200);
}

// ============================================================================
// Vertical Divider Tests
// ============================================================================

#[test]
fn test_divider_vertical_minimal_area() {
    let divider = Divider::vertical();
    let area = Rect::new(0, 0, 1, 5);
    let plane = divider.render(area);
    assert!(plane.height >= 5);
}

#[test]
fn test_divider_vertical_with_label() {
    let divider = Divider::vertical().with_label("V");
    let area = Rect::new(0, 0, 1, 30);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_vertical_styles() {
    let styles = vec![
        DividerStyle::Solid,
        DividerStyle::Dashed,
        DividerStyle::Double,
        DividerStyle::Bold,
    ];

    for style in styles {
        let divider = Divider::vertical().style(style);
        let area = Rect::new(0, 0, 1, 30);
        let plane = divider.render(area);
        assert!(plane.height > 0);
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_divider_empty_label() {
    let divider = Divider::new().with_label("");
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_long_label() {
    let long_label = "A".repeat(100);
    let divider = Divider::new().with_label(&long_label);
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_unicode_label() {
    let divider = Divider::new().with_label("日本語 | العربية");
    let area = Rect::new(0, 0, 80, 1);
    let _plane = divider.render(area);
}

#[test]
fn test_divider_combined_options() {
    let divider = Divider::new()
        .direction(DividerDirection::Horizontal)
        .style(DividerStyle::Double)
        .with_label("Combined")
        .label_position(LabelPosition::Center)
        .with_theme(Theme::nord());
    let area = Rect::new(0, 0, 80, 1);
    let plane = divider.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// Enums Test
// ============================================================================

#[test]
fn test_divider_direction_enum() {
    let directions = vec![DividerDirection::Horizontal, DividerDirection::Vertical];
    assert_eq!(directions.len(), 2);
}

#[test]
fn test_divider_style_enum() {
    let styles = vec![
        DividerStyle::Solid,
        DividerStyle::Dashed,
        DividerStyle::Double,
        DividerStyle::Bold,
    ];
    assert_eq!(styles.len(), 4);
}

#[test]
fn test_label_position_enum() {
    let positions = vec![
        LabelPosition::Left,
        LabelPosition::Center,
        LabelPosition::Right,
    ];
    assert_eq!(positions.len(), 3);
}

// ============================================================================
// Comparison Tests
// ============================================================================

// Clone and Debug tests removed - Divider doesn't derive Clone or Debug

// ============================================================================
// Complex Scenarios
// ============================================================================

#[test]
fn test_divider_multiple_combinations() {
    let directions = vec![DividerDirection::Horizontal, DividerDirection::Vertical];
    let styles = vec![
        DividerStyle::Solid,
        DividerStyle::Dashed,
        DividerStyle::Double,
        DividerStyle::Bold,
    ];
    let positions = vec![
        LabelPosition::Left,
        LabelPosition::Center,
        LabelPosition::Right,
    ];

    for dir in &directions {
        for style in &styles {
            for pos in &positions {
                let divider = Divider::new()
                    .direction(*dir)
                    .style(*style)
                    .with_label("Combo")
                    .label_position(*pos);
                let area = if matches!(dir, DividerDirection::Horizontal) {
                    Rect::new(0, 0, 80, 1)
                } else {
                    Rect::new(0, 0, 1, 30)
                };
                let _plane = divider.render(area);
            }
        }
    }
}

#[test]
fn test_divider_toggle_label() {
    let mut divider = Divider::new();
    divider.set_label(Some("Label 1"));
    let area1 = Rect::new(0, 0, 80, 1);
    let _plane1 = divider.render(area1);

    divider.set_label(Some("Label 2"));
    let area2 = Rect::new(0, 0, 80, 1);
    let _plane2 = divider.render(area2);

    divider.set_label(None);
    let area3 = Rect::new(0, 0, 80, 1);
    let _plane3 = divider.render(area3);
}

// ============================================================================
// Area Sensitivity Tests
// ============================================================================

#[test]
fn test_divider_narrow_width() {
    let divider = Divider::new();
    let area = Rect::new(0, 0, 1, 1);
    let plane = divider.render(area);
    assert_eq!(plane.width, 1);
}

#[test]
fn test_divider_full_width() {
    let divider = Divider::new();
    let area = Rect::new(0, 0, 300, 1);
    let plane = divider.render(area);
    assert_eq!(plane.width, 300);
}

#[test]
fn test_divider_short_vertical() {
    let divider = Divider::vertical();
    let area = Rect::new(0, 0, 1, 1);
    let plane = divider.render(area);
    assert!(plane.height >= 1);
}

#[test]
fn test_divider_tall_vertical() {
    let divider = Divider::vertical();
    let area = Rect::new(0, 0, 1, 100);
    let plane = divider.render(area);
    assert!(plane.height >= 100);
}
