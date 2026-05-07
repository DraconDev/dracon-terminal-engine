//! Accessibility tests — high contrast themes, screen reader hints.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Button, Checkbox, Label, List, Toggle};
use ratatui::layout::Rect;

#[allow(dead_code)]
/// Verify theme has sufficient contrast between fg and bg colors.
fn has_sufficient_contrast(fg: Color, bg: Color) -> bool {
    // Simple check: they should not be the same
    // In practice, this would use WCAG contrast ratios
    fg != bg
}

#[test]
fn test_high_contrast_themes_exist() {
    let themes = vec![Theme::dark(), Theme::light()];

    for theme in themes {
        // fg and bg should be different
        assert_ne!(
            theme.fg, theme.bg,
            "Theme {} has insufficient fg/bg contrast",
            theme.name
        );
    }
}

#[test]
fn test_high_contrast_theme_rendering() {
    let theme = Theme::light();

    let mut btn = Button::with_id(WidgetId::new(1), "Click");
    btn.on_theme_change(&theme);
    let plane = btn.render(Rect::new(0, 0, 15, 3));

    // All cells should be visible (not Color::Reset)
    for cell in &plane.cells {
        assert_ne!(cell.bg, Color::Reset);
    }
}

#[test]
fn test_button_focus_indicator() {
    let mut btn = Button::with_id(WidgetId::new(1), "Click");
    btn.on_theme_change(&Theme::nord());

    // Focus should not panic
    btn.on_focus();
    let plane_focused = btn.render(Rect::new(0, 0, 15, 3));

    // Blur should not panic
    btn.on_blur();
    let plane_blurred = btn.render(Rect::new(0, 0, 15, 3));

    // Both should render without issue
    assert!(!plane_focused.cells.is_empty());
    assert!(!plane_blurred.cells.is_empty());
}

#[test]
fn test_checkbox_checked_state_visible() {
    let mut cb = Checkbox::new(WidgetId::new(1), "Option");
    cb.on_theme_change(&Theme::nord());

    let plane_unchecked = cb.render(Rect::new(0, 0, 20, 1));

    cb.check();
    let plane_checked = cb.render(Rect::new(0, 0, 20, 1));

    // Checked and unchecked should look different
    assert_ne!(plane_unchecked.cells, plane_checked.cells);
}

#[test]
fn test_toggle_on_off_visible() {
    let mut toggle = Toggle::new(WidgetId::new(1), "Dark Mode");
    toggle.on_theme_change(&Theme::nord());

    let plane_off = toggle.render(Rect::new(0, 0, 20, 1));

    // Toggle on
    toggle.handle_key(KeyEvent {
        code: KeyCode::Enter,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    });

    let plane_on = toggle.render(Rect::new(0, 0, 20, 1));

    // On and off should look different
    assert_ne!(plane_off.cells, plane_on.cells);
}

#[test]
fn test_list_selection_visible() {
    let items = vec!["A".to_string(), "B".to_string(), "C".to_string()];
    let mut list = List::new_with_id(WidgetId::new(1), items);
    list.on_theme_change(&Theme::nord());

    // Render without selection
    let plane_no_sel = list.render(Rect::new(0, 0, 20, 5));

    // Select an item using keyboard
    list.handle_key(KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    });
    let plane_sel = list.render(Rect::new(0, 0, 20, 5));

    // Should be different
    assert_ne!(plane_no_sel.cells, plane_sel.cells);
}

#[test]
fn test_label_text_readable() {
    let mut label = Label::new("Important Message");
    label.on_theme_change(&Theme::nord());

    let plane = label.render(Rect::new(0, 0, 30, 1));

    // Should have visible text
    let has_visible_chars = plane.cells.iter().any(|c| c.char != ' ' && c.char != '\0');
    assert!(has_visible_chars);
}

#[test]
fn test_all_themes_have_visible_fg() {
    let themes = vec![
        Theme::dark(),
        Theme::light(),
        Theme::cyberpunk(),
        Theme::dracula(),
        Theme::nord(),
        Theme::catppuccin_mocha(),
        Theme::gruvbox_dark(),
        Theme::tokyo_night(),
        Theme::solarized_dark(),
        Theme::solarized_light(),
        Theme::one_dark(),
        Theme::rose_pine(),
        Theme::kanagawa(),
        Theme::everforest(),
        Theme::monokai(),
        Theme::warm(),
        Theme::cool(),
        Theme::forest(),
        Theme::sunset(),
        Theme::mono(),
    ];

    for theme in &themes {
        // fg should not be Reset (invisible)
        assert_ne!(
            theme.fg,
            Color::Reset,
            "Theme {} has invisible fg",
            theme.name
        );
        assert_ne!(
            theme.bg,
            Color::Reset,
            "Theme {} has invisible bg",
            theme.name
        );
    }
}

#[test]
fn test_theme_colors_are_distinct() {
    let themes = vec![
        Theme::dark(),
        Theme::light(),
        Theme::cyberpunk(),
        Theme::dracula(),
        Theme::nord(),
    ];

    for theme in &themes {
        // Primary colors should be different from bg
        assert_ne!(
            theme.primary, theme.bg,
            "Theme {} primary same as bg",
            theme.name
        );
        assert_ne!(
            theme.error, theme.bg,
            "Theme {} error same as bg",
            theme.name
        );
    }
}
