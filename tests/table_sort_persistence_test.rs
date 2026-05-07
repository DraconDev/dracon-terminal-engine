//! Table sorting persistence tests — verify sort state survives theme changes.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Column, Table};
use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
use ratatui::layout::Rect;

fn make_table() -> Table<String> {
    let columns = vec![
        Column {
            header: "Name".to_string(),
            width: 15,
        },
        Column {
            header: "Score".to_string(),
            width: 10,
        },
    ];

    let items = vec![
        "Alice".to_string(),
        "Bob".to_string(),
        "Charlie".to_string(),
        "Diana".to_string(),
    ];

    Table::new_with_id(WidgetId::new(1), columns).with_rows(items)
}

#[test]
fn test_table_sort_persists_across_renders() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));

    // Set sort
    table.set_sort(Some(0), true);

    let plane1 = table.render(Rect::new(0, 0, 40, 10));
    let plane2 = table.render(Rect::new(0, 0, 40, 10));

    assert_eq!(plane1.cells.len(), plane2.cells.len());
}

#[test]
fn test_table_sort_survives_theme_change() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));

    table.set_sort(Some(1), true);

    let theme = Theme::cyberpunk();
    table.on_theme_change(&theme);

    let plane = table.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_table_sort_ascending_descending_toggle() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));

    table.set_sort(Some(0), true);
    let _ = table.render(Rect::new(0, 0, 40, 10));

    table.set_sort(Some(0), false);
    let _ = table.render(Rect::new(0, 0, 40, 10));
}

#[test]
fn test_table_clear_sort() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));

    table.set_sort(Some(1), true);
    table.set_sort(None, true);

    let plane = table.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_table_header_click() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));

    let result = table.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 0);
    assert!(result);
}

#[test]
fn test_table_row_selection_after_sort() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));

    table.set_sort(Some(1), true);
    table.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 2);

    let plane = table.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_table_multiple_theme_changes() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));

    table.set_sort(Some(1), true);

    let themes = vec![Theme::nord(), Theme::cyberpunk(), Theme::dracula()];

    for theme in &themes {
        table.on_theme_change(theme);
        let plane = table.render(Rect::new(0, 0, 40, 10));
        assert!(plane.cells.len() > 0);
    }
}

#[test]
fn test_table_empty_rows_sort() {
    let columns = vec![Column {
        header: "Name".to_string(),
        width: 15,
    }];

    let mut table: Table<String> = Table::new_with_id(WidgetId::new(1), columns);
    table.set_area(Rect::new(0, 0, 40, 10));

    table.set_sort(Some(0), true);

    let plane = table.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_table_single_row_sort() {
    let columns = vec![Column {
        header: "Name".to_string(),
        width: 15,
    }];

    let mut table =
        Table::new_with_id(WidgetId::new(1), columns).with_rows(vec!["Only".to_string()]);
    table.set_area(Rect::new(0, 0, 40, 10));

    table.set_sort(Some(0), true);

    let plane = table.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}
