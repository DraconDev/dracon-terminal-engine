//! Tests for the Select widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::select::Select;
use ratatui::layout::Rect;

fn make_select(options: Vec<String>) -> Select {
    Select::new(WidgetId::new(0)).with_options(options)
}

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_select_new() {
    let select = Select::new(WidgetId::new(42));
    let area = Rect::new(0, 0, 20, 1);
    let plane = select.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_select_new_with_options() {
    let options = vec!["Option 1".to_string(), "Option 2".to_string(), "Option 3".to_string()];
    let select = make_select(options);
    assert_eq!(select.selected_index(), 0);
}

#[test]
fn test_select_empty_options() {
    let select = make_select(vec![]);
    let area = Rect::new(0, 0, 20, 1);
    let plane = select.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_select_single_option() {
    let options = vec!["Only".to_string()];
    let select = make_select(options);
    assert_eq!(select.selected_index(), 0);
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_select_with_theme() {
    let options = vec!["Test".to_string()];
    let select = make_select(options).with_theme(Theme::nord());
    let area = Rect::new(0, 0, 20, 10);
    let _plane = select.render(area);
}

#[test]
fn test_select_on_change_callback() {
    use std::cell::RefCell;
    use std::rc::Rc;
    
    let changed = Rc::new(RefCell::new(Vec::new()));
    let changed_clone = Rc::clone(&changed);
    
    let options = vec!["A".to_string(), "B".to_string()];
    let _select = make_select(options)
        .on_change(move |label| {
            changed_clone.borrow_mut().push(label.to_string());
        });
}

#[test]
fn test_select_chained_builders() {
    let options = vec!["Rust".to_string(), "Go".to_string()];
    let select = make_select(options)
        .with_theme(Theme::cyberpunk())
        .on_change(|_| {});
    
    let area = Rect::new(0, 0, 20, 10);
    let _plane = select.render(area);
}

// ============================================================================
// Selection Tests
// ============================================================================

#[test]
fn test_select_default_selected_0() {
    let options = vec!["One".to_string(), "Two".to_string()];
    let select = make_select(options);
    assert_eq!(select.selected_index(), 0);
}

#[test]
fn test_select_set_selected_valid() {
    let mut select = make_select(vec![
        "First".to_string(),
        "Second".to_string(),
        "Third".to_string(),
    ]);
    select.set_selected(1);
    assert_eq!(select.selected_index(), 1);
}

#[test]
fn test_select_set_selected_out_of_bounds() {
    let mut select = make_select(vec!["Only One".to_string()]);
    select.set_selected(100);
    // Should clamp to valid index
    assert_eq!(select.selected_index(), 0);
}

#[test]
fn test_select_set_selected_empty() {
    let mut select = make_select(vec![]);
    select.set_selected(0);
    assert_eq!(select.selected_index(), 0);
}

#[test]
fn test_select_default_label() {
    let select = make_select(vec!["Test".to_string()]);
    let label = select.selected_label();
    assert_eq!(label, Some("Test"));
}

#[test]
fn test_select_empty_options_label() {
    let select = make_select(vec![]);
    let label = select.selected_label();
    assert_eq!(label, None);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_select_id() {
    let options = vec!["Test".to_string()];
    let select = make_select(options);
    let _id = select.id();
}

#[test]
fn test_select_area() {
    let select = make_select(vec!["Test".to_string()]);
    let area = select.area();
    assert!(area.width > 0);
}

#[test]
fn test_select_set_area() {
    let mut select = make_select(vec!["Test".to_string()]);
    let new_area = Rect::new(10, 20, 40, 5);
    select.set_area(new_area);
    assert_eq!(select.area(), new_area);
}

#[test]
fn test_select_needs_render() {
    let select = make_select(vec!["Test".to_string()]);
    assert!(select.needs_render());
}

#[test]
fn test_select_mark_dirty() {
    let mut select = make_select(vec!["Test".to_string()]);
    select.clear_dirty();
    assert!(!select.needs_render());
    select.mark_dirty();
    assert!(select.needs_render());
}

#[test]
fn test_select_clear_dirty() {
    let mut select = make_select(vec!["Test".to_string()]);
    select.clear_dirty();
    assert!(!select.needs_render());
}

#[test]
fn test_select_render() {
    let select = make_select(vec!["Test".to_string()]);
    let area = Rect::new(0, 0, 20, 1);
    let plane = select.render(area);
    assert_eq!(plane.width, 20);
}

#[test]
fn test_select_render_expanded() {
    let mut select = make_select(vec!["Opt 1".to_string(), "Opt 2".to_string(), "Opt 3".to_string()]);
    // Expanded state is internal, just verify render works
    let area = Rect::new(0, 0, 20, 20);
    let plane = select.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_select_z_index() {
    let select = make_select(vec!["Test".to_string()]);
    let _z = select.z_index();
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_select_different_themes() {
    let options = vec!["Test".to_string()];
    let themes = vec!["nord", "dracula", "monokai", "solarized_dark"];
    
    for theme_name in themes {
        if let Some(theme) = Theme::from_name(theme_name) {
            let select = make_select(options.clone()).with_theme(theme);
            let area = Rect::new(0, 0, 20, 10);
            let plane = select.render(area);
            assert!(plane.width > 0);
        }
    }
}

#[test]
fn test_select_on_theme_change() {
    let mut select = make_select(vec!["Test".to_string()]);
    select.on_theme_change(&Theme::nord());
    assert!(select.needs_render());
}

// ============================================================================
// Rendering Tests
// ============================================================================

#[test]
fn test_select_render_fills_bg() {
    let select = make_select(vec!["Test".to_string()]);
    let area = Rect::new(0, 0, 20, 10);
    let plane = select.render(area);
    let theme = Theme::default();
    assert_eq!(plane.cells[0].bg, theme.bg);
}

#[test]
fn test_select_render_has_content() {
    let select = make_select(vec!["Test".to_string()]);
    let area = Rect::new(0, 0, 20, 1);
    let plane = select.render(area);
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content, "Select should render some content");
}

#[test]
fn test_select_render_minimal_area() {
    let select = make_select(vec!["Test".to_string()]);
    let area = Rect::new(0, 0, 5, 1);
    let plane = select.render(area);
    assert_eq!(plane.width, 5);
}

#[test]
fn test_select_render_wide_area() {
    let select = make_select(vec!["Test".to_string()]);
    let area = Rect::new(0, 0, 100, 1);
    let plane = select.render(area);
    assert_eq!(plane.width, 100);
}

// ============================================================================
// Options Tests
// ============================================================================

#[test]
fn test_select_many_options() {
    let options: Vec<String> = (0..50).map(|i| format!("Option {}", i)).collect();
    let select = make_select(options);
    let area = Rect::new(0, 0, 20, 20);
    let _plane = select.render(area);
}

#[test]
fn test_select_unicode_options() {
    let options = vec![
        "日本語".to_string(),
        "العربية".to_string(),
        "🎉".to_string(),
    ];
    let select = make_select(options);
    let area = Rect::new(0, 0, 20, 10);
    let _plane = select.render(area);
}

#[test]
fn test_select_long_options() {
    let long_text = "A".repeat(100);
    let options = vec![long_text.to_string()];
    let select = make_select(options);
    let area = Rect::new(0, 0, 20, 10);
    let _plane = select.render(area);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_select_empty_string_option() {
    let options = vec!["".to_string()];
    let select = make_select(options);
    let area = Rect::new(0, 0, 20, 1);
    let _plane = select.render(area);
}

#[test]
fn test_select_all_empty_options() {
    let options = vec!["".to_string(), "".to_string(), "".to_string()];
    let select = make_select(options);
    let area = Rect::new(0, 0, 20, 1);
    let _plane = select.render(area);
}

#[test]
fn test_select_multiple_set_selected() {
    let mut select = make_select(vec!["A".to_string(), "B".to_string(), "C".to_string()]);
    
    select.set_selected(0);
    assert_eq!(select.selected_index(), 0);
    
    select.set_selected(1);
    assert_eq!(select.selected_index(), 1);
    
    select.set_selected(2);
    assert_eq!(select.selected_index(), 2);
}

#[test]
fn test_select_with_many_themes() {
    let options = vec!["Test".to_string()];
    let themes = vec![
        "nord", "dracula", "monokai", "solarized_dark", "catppuccin_mocha",
        "tokyo_night", "gruvbox_dark", "ayu_dark", "material_darker",
    ];
    
    for theme_name in themes {
        if let Some(theme) = Theme::from_name(theme_name) {
            let select = make_select(options.clone()).with_theme(theme);
            let area = Rect::new(0, 0, 20, 10);
            let _plane = select.render(area);
        }
    }
}

#[test]
fn test_select_clamp_to_bounds() {
    let options = vec!["First".to_string(), "Second".to_string()];
    let mut select = make_select(options);
    
    // Try to set beyond bounds
    select.set_selected(100);
    assert_eq!(select.selected_index(), 1);
    
    // Try very large number (should clamp)
    select.set_selected(u8::MAX as usize);
    assert_eq!(select.selected_index(), 1);
}

#[test]
fn test_select_clear_dirty_then_mark() {
    let mut select = make_select(vec!["Test".to_string()]);
    select.clear_dirty();
    assert!(!select.needs_render());
    select.mark_dirty();
    assert!(select.needs_render());
}

#[test]
fn test_select_render_small_area() {
    let select = make_select(vec!["Test".to_string()]);
    let area = Rect::new(0, 0, 1, 1);
    let plane = select.render(area);
    assert_eq!(plane.width, 1);
}

#[test]
fn test_select_render_tall_area() {
    let select = make_select(vec!["Test".to_string()]);
    let area = Rect::new(0, 0, 20, 50);
    let plane = select.render(area);
    assert_eq!(plane.height, 50);
}