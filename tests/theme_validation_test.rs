//! Theme validation tests — verify widgets render with theme.bg backgrounds,
//! not Color::Reset (which appears black).

use dracon_terminal_engine::compositor::{Color, Plane};
use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{
    Button, Checkbox, Column, Label, List, ProgressBar, Radio, SearchInput, Select, Slider, Spinner,
    Table, Toggle,
};
use ratatui::layout::Rect;

/// All built-in themes to test against.
fn all_themes() -> Vec<Theme> {
    vec![
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
    ]
}

/// Assert that every cell in the plane has bg != Color::Reset.
fn assert_no_black_background(plane: &Plane, widget_name: &str) {
    let black_cells: Vec<usize> = plane
        .cells
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.bg == Color::Reset)
        .map(|(idx, _)| idx)
        .collect();

    assert!(
        black_cells.is_empty(),
        "{}: found {} cells with Color::Reset (black) background at indices {:?}",
        widget_name,
        black_cells.len(),
        &black_cells[..black_cells.len().min(10)]
    );
}

#[test]
fn test_checkbox_no_black_background() {
    for theme in all_themes() {
        let mut cb = Checkbox::new(WidgetId::new(1), "Test");
        cb.on_theme_change(&theme);
        let plane = cb.render(Rect::new(0, 0, 20, 1));
        assert_no_black_background(&plane, "Checkbox");
    }
}

#[test]
fn test_button_no_black_background() {
    for theme in all_themes() {
        let mut btn = Button::with_id(WidgetId::new(1), "Click");
        btn.on_theme_change(&theme);
        let plane = btn.render(Rect::new(0, 0, 15, 1));
        assert_no_black_background(&plane, "Button");
    }
}

#[test]
fn test_label_no_black_background() {
    for theme in all_themes() {
        let mut label = Label::new("Hello");
        label.on_theme_change(&theme);
        let plane = label.render(Rect::new(0, 0, 10, 1));
        assert_no_black_background(&plane, "Label");
    }
}

#[test]
fn test_toggle_no_black_background() {
    for theme in all_themes() {
        let mut toggle = Toggle::new(WidgetId::new(1), "Dark");
        toggle.on_theme_change(&theme);
        let plane = toggle.render(Rect::new(0, 0, 20, 1));
        assert_no_black_background(&plane, "Toggle");
    }
}

#[test]
fn test_spinner_no_black_background() {
    for theme in all_themes() {
        let mut spinner = Spinner::new(WidgetId::new(1));
        spinner.on_theme_change(&theme);
        let plane = spinner.render(Rect::new(0, 0, 10, 1));
        assert_no_black_background(&plane, "Spinner");
    }
}

#[test]
fn test_progress_bar_no_black_background() {
    for theme in all_themes() {
        let mut pb = ProgressBar::new(WidgetId::new(1));
        pb.on_theme_change(&theme);
        let plane = pb.render(Rect::new(0, 0, 30, 1));
        assert_no_black_background(&plane, "ProgressBar");
    }
}

#[test]
fn test_list_no_black_background() {
    for theme in all_themes() {
        let items = vec!["Apple".to_string(), "Banana".to_string(), "Cherry".to_string()];
        let mut list = List::new(WidgetId::new(1), items);
        list.on_theme_change(&theme);
        let plane = list.render(Rect::new(0, 0, 20, 5));
        assert_no_black_background(&plane, "List");
    }
}

#[test]
fn test_table_no_black_background() {
    for theme in all_themes() {
        let columns = vec![
            Column { header: "Name".to_string(), width: 20 },
            Column { header: "Age".to_string(), width: 10 },
        ];
        let mut table: Table<String> = Table::new_with_id(WidgetId::new(1), columns);
        table.on_theme_change(&theme);
        let plane = table.render(Rect::new(0, 0, 40, 4));
        assert_no_black_background(&plane, "Table");
    }
}

#[test]
fn test_all_20_themes_no_panic() {
    // Sanity check: rendering every widget with every theme doesn't panic
    let themes = all_themes();
    assert_eq!(themes.len(), 20, "Expected 20 themes");
    
    for theme in &themes {
        let mut cb = Checkbox::new(WidgetId::new(1), "Test");
        cb.on_theme_change(theme);
        let _ = cb.render(Rect::new(0, 0, 20, 1));
        
        let mut btn = Button::with_id(WidgetId::new(1), "Click");
        btn.on_theme_change(theme);
        let _ = btn.render(Rect::new(0, 0, 15, 1));
        
        let mut list = List::new(vec!["a".to_string()]);
        list.on_theme_change(theme);
        let _ = list.render(Rect::new(0, 0, 20, 3));
    }
}

#[test]
fn test_select_no_black_background() {
    for theme in all_themes() {
        let mut select = Select::new(WidgetId::new(1))
            .with_options(vec!["A".to_string(), "B".to_string()]);
        select.on_theme_change(&theme);
        let plane = select.render(Rect::new(0, 0, 20, 1));
        assert_no_black_background(&plane, "Select");
    }
}

#[test]
fn test_slider_no_black_background() {
    for theme in all_themes() {
        let mut slider = Slider::new(WidgetId::new(1)).with_range(0.0, 100.0);
        slider.on_theme_change(&theme);
        let plane = slider.render(Rect::new(0, 0, 30, 1));
        assert_no_black_background(&plane, "Slider");
    }
}

#[test]
fn test_radio_no_black_background() {
    for theme in all_themes() {
        let mut radio = Radio::new(WidgetId::new(1), "Option");
        radio.on_theme_change(&theme);
        let plane = radio.render(Rect::new(0, 0, 20, 1));
        assert_no_black_background(&plane, "Radio");
    }
}

#[test]
fn test_search_input_no_black_background() {
    for theme in all_themes() {
        let mut search = SearchInput::new(WidgetId::new(1));
        search.on_theme_change(&theme);
        let plane = search.render(Rect::new(0, 0, 30, 1));
        assert_no_black_background(&plane, "SearchInput");
    }
}

#[test]
fn test_all_20_themes_no_panic() {
    // Sanity check: rendering every widget with every theme doesn't panic
    let themes = all_themes();
    assert_eq!(themes.len(), 20, "Expected 20 themes");
    
    for theme in &themes {
        let mut cb = Checkbox::new(WidgetId::new(1), "Test");
        cb.on_theme_change(theme);
        let _ = cb.render(Rect::new(0, 0, 20, 1));
        
        let mut btn = Button::with_id(WidgetId::new(1), "Click");
        btn.on_theme_change(theme);
        let _ = btn.render(Rect::new(0, 0, 15, 1));
        
        let mut list = List::new(vec!["a".to_string()]);
        list.on_theme_change(theme);
        let _ = list.render(Rect::new(0, 0, 20, 3));
    }
}
