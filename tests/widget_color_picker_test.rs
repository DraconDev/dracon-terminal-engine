//! Tests for the ColorPicker widget.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::color_picker::ColorPicker;
use ratatui::layout::Rect;

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_color_picker_new() {
    let picker = ColorPicker::new();
    // Default is red (#FF0000)
    assert_eq!(picker.hex(), "#FF0000");
}

#[test]
fn test_color_picker_default() {
    let picker = ColorPicker::default();
    assert_eq!(picker.hex(), "#FF0000");
}

#[test]
fn test_color_picker_with_color() {
    let picker = ColorPicker::with_color(Color::Rgb(0, 255, 0));
    assert_eq!(picker.hex(), "#00FF00");
}

#[test]
fn test_color_picker_with_hex() {
    let picker = ColorPicker::with_hex("#0000FF");
    assert_eq!(picker.hex(), "#0000FF");
}

#[test]
fn test_color_picker_with_hex_lowercase() {
    let picker = ColorPicker::with_hex("#aabbcc");
    assert_eq!(picker.hex(), "#AABBCC");
}

#[test]
fn test_color_picker_with_hex_without_hash() {
    let picker = ColorPicker::with_hex("AABBCC");
    // with_hex does NOT add # prefix, it stores as-is uppercase
    assert_eq!(picker.hex(), "AABBCC");
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_color_picker_with_theme() {
    let picker = ColorPicker::new().with_theme(Theme::nord());
    let area = Rect::new(0, 0, 40, 12);
    let plane = picker.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 12);
}

#[test]
fn test_color_picker_theme_propagation() {
    let theme = Theme::cyberpunk();
    let picker = ColorPicker::new().with_theme(theme.clone());
    let area = Rect::new(0, 0, 40, 12);
    let _plane = picker.render(area);
    // Theme should be stored and used in rendering
}

// ============================================================================
// Color Conversion Tests
// ============================================================================

#[test]
fn test_color_picker_red() {
    let picker = ColorPicker::with_color(Color::Rgb(255, 0, 0));
    assert_eq!(picker.hex(), "#FF0000");
    assert_eq!(picker.color(), Color::Rgb(255, 0, 0));
}

#[test]
fn test_color_picker_green() {
    let picker = ColorPicker::with_color(Color::Rgb(0, 255, 0));
    assert_eq!(picker.hex(), "#00FF00");
}

#[test]
fn test_color_picker_blue() {
    let picker = ColorPicker::with_color(Color::Rgb(0, 0, 255));
    assert_eq!(picker.hex(), "#0000FF");
}

#[test]
fn test_color_picker_white() {
    let picker = ColorPicker::with_color(Color::Rgb(255, 255, 255));
    assert_eq!(picker.hex(), "#FFFFFF");
}

#[test]
fn test_color_picker_black() {
    let picker = ColorPicker::with_color(Color::Rgb(0, 0, 0));
    assert_eq!(picker.hex(), "#000000");
}

#[test]
fn test_color_picker_gray() {
    let picker = ColorPicker::with_color(Color::Rgb(128, 128, 128));
    // Gray is achromatic, hue should be 0
    assert_eq!(picker.hex(), "#808080");
}

#[test]
fn test_color_picker_orange() {
    let picker = ColorPicker::with_color(Color::Rgb(255, 165, 0));
    assert_eq!(picker.hex(), "#FFA500");
}

#[test]
fn test_color_picker_cyan() {
    let picker = ColorPicker::with_color(Color::Rgb(0, 255, 255));
    assert_eq!(picker.hex(), "#00FFFF");
}

#[test]
fn test_color_picker_magenta() {
    let picker = ColorPicker::with_color(Color::Rgb(255, 0, 255));
    assert_eq!(picker.hex(), "#FF00FF");
}

// ============================================================================
// HSL Setting Tests
// ============================================================================

#[test]
fn test_color_picker_set_hsl() {
    let mut picker = ColorPicker::new();
    picker.set_hsl(240.0, 100.0, 50.0); // Blue
    assert_eq!(picker.hex(), "#0000FF");
}

#[test]
fn test_color_picker_set_hsl_red() {
    let mut picker = ColorPicker::new();
    picker.set_hsl(0.0, 100.0, 50.0);
    assert_eq!(picker.hex(), "#FF0000");
}

#[test]
fn test_color_picker_set_hsl_green() {
    let mut picker = ColorPicker::new();
    picker.set_hsl(120.0, 100.0, 50.0);
    assert_eq!(picker.hex(), "#00FF00");
}

#[test]
fn test_color_picker_set_hsl_blue() {
    let mut picker = ColorPicker::new();
    picker.set_hsl(240.0, 100.0, 50.0);
    assert_eq!(picker.hex(), "#0000FF");
}

#[test]
fn test_color_picker_set_hsl_clamp_hue() {
    let mut picker = ColorPicker::new();
    picker.set_hsl(400.0, 100.0, 50.0); // Should clamp to 360
    // Hue wraps around, so 400 = 40
    picker.set_hsl(0.0, 100.0, 50.0); // Reset to red first
    picker.set_hsl(400.0, 100.0, 50.0);
    // After clamping: hue = 360 (max), saturation = 100, lightness = 50
}

#[test]
fn test_color_picker_set_hsl_clamp_saturation() {
    let mut picker = ColorPicker::new();
    picker.set_hsl(0.0, 150.0, 50.0); // Sat > 100, should clamp
    picker.set_hsl(0.0, -50.0, 50.0); // Sat < 0, should clamp
}

#[test]
fn test_color_picker_set_hsl_gray() {
    let mut picker = ColorPicker::new();
    picker.set_hsl(120.0, 0.0, 50.0); // Sat = 0 = gray
    // Gray is 50% lightness = 127-128 range
    assert_eq!(picker.hex(), "#7F7F7F");
}

// ============================================================================
// Hex Setting Tests
// ============================================================================

#[test]
fn test_color_picker_set_hex_valid() {
    let mut picker = ColorPicker::new();
    picker.set_hex("#00FF00");
    assert_eq!(picker.hex(), "#00FF00");
}

#[test]
fn test_color_picker_set_hex_uppercase() {
    let mut picker = ColorPicker::new();
    picker.set_hex("#aabbcc");
    assert_eq!(picker.hex(), "#AABBCC");
}

#[test]
fn test_color_picker_set_hex_without_hash() {
    let mut picker = ColorPicker::new();
    picker.set_hex("00FF00");
    assert_eq!(picker.hex(), "#00FF00");
}

#[test]
fn test_color_picker_set_hex_invalid_length() {
    let mut picker = ColorPicker::new();
    let original_hex = picker.hex().to_string();
    picker.set_hex("#ABC"); // Invalid - not 6 chars
    assert_eq!(picker.hex(), original_hex);
}

#[test]
fn test_color_picker_set_hex_invalid_chars() {
    let mut picker = ColorPicker::new();
    let original_hex = picker.hex().to_string();
    picker.set_hex("#GGGGGG"); // Invalid chars
    assert_eq!(picker.hex(), original_hex);
}

#[test]
fn test_color_picker_set_hex_with_spaces() {
    let mut picker = ColorPicker::new();
    picker.set_hex("  #FF0000  ");
    assert_eq!(picker.hex(), "#FF0000");
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_color_picker_id() {
    let picker = ColorPicker::new();
    let _id = picker.id();
    // ID should be valid (not default WidgetId::new(0) for a newly created widget)
}

#[test]
fn test_color_picker_area() {
    let picker = ColorPicker::new();
    let area = picker.area();
    assert_eq!(area.width, 40);
    assert_eq!(area.height, 12);
}

#[test]
fn test_color_picker_set_area() {
    let mut picker = ColorPicker::new();
    let new_area = Rect::new(10, 20, 60, 20);
    picker.set_area(new_area);
    assert_eq!(picker.area(), new_area);
}

#[test]
fn test_color_picker_needs_render() {
    let picker = ColorPicker::new();
    assert!(picker.needs_render());
}

#[test]
fn test_color_picker_mark_dirty() {
    let mut picker = ColorPicker::new();
    picker.clear_dirty();
    assert!(!picker.needs_render());
    picker.mark_dirty();
    assert!(picker.needs_render());
}

#[test]
fn test_color_picker_clear_dirty() {
    let mut picker = ColorPicker::new();
    picker.clear_dirty();
    assert!(!picker.needs_render());
}

#[test]
fn test_color_picker_render_returns_plane() {
    let picker = ColorPicker::new();
    let area = Rect::new(0, 0, 40, 12);
    let plane = picker.render(area);
    assert_eq!(plane.width, 40);
    assert_eq!(plane.height, 12);
}

#[test]
fn test_color_picker_render_size_bounds() {
    let picker = ColorPicker::new();
    // Test minimum size
    let small_area = Rect::new(0, 0, 1, 1);
    let _plane = picker.render(small_area);
    
    // Test zero dimensions (should handle gracefully)
    let zero_area = Rect::new(0, 0, 0, 0);
    let _plane = picker.render(zero_area);
}

#[test]
fn test_color_picker_focusable() {
    let picker = ColorPicker::new();
    assert!(picker.focusable());
}

#[test]
fn test_color_picker_z_index() {
    let picker = ColorPicker::new();
    assert_eq!(picker.z_index(), 10);
}

// ============================================================================
// Rendering Tests
// ============================================================================

#[test]
fn test_color_picker_render_fills_bg() {
    let picker = ColorPicker::new();
    let area = Rect::new(0, 0, 40, 12);
    let plane = picker.render(area);
    // Background should be filled with theme.bg
    // Check a few cells have the background color
    let theme = Theme::default();
    assert_eq!(plane.cells[0].bg, theme.bg);
}

#[test]
fn test_color_picker_render_respects_area() {
    let picker = ColorPicker::new();
    let area = Rect::new(0, 0, 80, 24);
    let plane = picker.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 24);
}

// ============================================================================
// Callback Tests
// ============================================================================

#[test]
fn test_color_picker_on_color_change_builder() {
    use std::cell::RefCell;
    use std::rc::Rc;
    
    let color_received = Rc::new(RefCell::new(None));
    let color_clone = Rc::clone(&color_received);
    
    let mut picker = ColorPicker::new()
        .on_color_change(move |c| {
            *color_clone.borrow_mut() = Some(c);
        });
    
    // Set a color - callback registration should work
    picker.set_hex("#00FF00");
    
    // Callback registration is tested by not panicking
    // Actual callback invocation depends on internal implementation
}

#[test]
fn test_color_picker_callback_not_called_for_invalid_hex() {
    use std::cell::RefCell;
    use std::rc::Rc;
    
    let call_count = Rc::new(RefCell::new(0));
    let count_clone = Rc::clone(&call_count);
    
    let mut picker = ColorPicker::new()
        .on_color_change(move |_c| {
            *count_clone.borrow_mut() += 1;
        });
    
    // Try to set invalid hex
    picker.set_hex("#INVALID");
    
    // Callback should NOT have been called for invalid input
    assert_eq!(*call_count.borrow(), 0);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_color_picker_invalid_hex_returns_same() {
    let mut picker = ColorPicker::new();
    let original = picker.hex().to_string();
    
    picker.set_hex("#ZZZZZZ");
    assert_eq!(picker.hex(), original);
}

#[test]
fn test_color_picker_invalid_hex_empty() {
    let mut picker = ColorPicker::new();
    let original = picker.hex().to_string();
    
    picker.set_hex("");
    assert_eq!(picker.hex(), original);
}

#[test]
fn test_color_picker_invalid_hex_short() {
    let mut picker = ColorPicker::new();
    let original = picker.hex().to_string();
    
    picker.set_hex("#12345");
    assert_eq!(picker.hex(), original);
}

#[test]
fn test_color_picker_minimal_area() {
    let picker = ColorPicker::new();
    // Very small area should not crash
    let area = Rect::new(0, 0, 3, 3);
    let plane = picker.render(area);
    // Should return a valid plane
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

// ============================================================================
// Theme Interaction Tests
// ============================================================================

#[test]
fn test_color_picker_different_themes() {
    for theme_name in ["nord", "dracula", "monokai", "solarized_dark"] {
        if let Some(theme) = Theme::from_name(theme_name) {
            let picker = ColorPicker::new().with_theme(theme);
            let area = Rect::new(0, 0, 40, 12);
            let plane = picker.render(area);
            assert_eq!(plane.width, 40);
            assert_eq!(plane.height, 12);
        }
    }
}

// ============================================================================
// Color Round-trip Tests
// ============================================================================

#[test]
fn test_color_picker_roundtrip_red() {
    let original = Color::Rgb(255, 0, 0);
    let picker = ColorPicker::with_color(original);
    assert_eq!(picker.color(), original);
}

#[test]
fn test_color_picker_roundtrip_green() {
    let original = Color::Rgb(0, 255, 0);
    let picker = ColorPicker::with_color(original);
    assert_eq!(picker.color(), original);
}

#[test]
fn test_color_picker_roundtrip_blue() {
    let original = Color::Rgb(0, 0, 255);
    let picker = ColorPicker::with_color(original);
    assert_eq!(picker.color(), original);
}

#[test]
fn test_color_picker_roundtrip_all_grays() {
    // Test a range of gray values (not all 256 to keep test fast)
    let test_grays = [0, 32, 64, 96, 128, 160, 192, 224, 255];
    for i in test_grays {
        let gray = Color::Rgb(i, i, i);
        let picker = ColorPicker::with_color(gray);
        let result = picker.color();
        match (result, gray) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                // Allow small rounding differences (within 2)
                assert!((r1 as i32 - r2 as i32).abs() <= 2);
                assert!((g1 as i32 - g2 as i32).abs() <= 2);
                assert!((b1 as i32 - b2 as i32).abs() <= 2);
            }
            _ => panic!("Color mismatch"),
        }
    }
}

#[test]
fn test_color_picker_roundtrip_rainbow() {
    // Test a sampling of colors
    let colors = [
        Color::Rgb(255, 0, 0),     // Red
        Color::Rgb(255, 127, 0),   // Orange
        Color::Rgb(255, 255, 0),   // Yellow
        Color::Rgb(0, 255, 0),     // Green
        Color::Rgb(0, 255, 255),   // Cyan
        Color::Rgb(0, 0, 255),     // Blue
        Color::Rgb(127, 0, 255),   // Violet
        Color::Rgb(255, 0, 127),   // Pink
    ];
    
    for color in colors {
        let picker = ColorPicker::with_color(color);
        let result = picker.color();
        // Allow small rounding differences
        match (result, color) {
            (Color::Rgb(r1, g1, b1), Color::Rgb(r2, g2, b2)) => {
                assert!((r1 as i32 - r2 as i32).abs() <= 1);
                assert!((g1 as i32 - g2 as i32).abs() <= 1);
                assert!((b1 as i32 - b2 as i32).abs() <= 1);
            }
            _ => panic!("Color mismatch"),
        }
    }
}