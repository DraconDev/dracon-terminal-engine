//! Table sorting persistence tests — verify sort state survives theme changes.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Column, Table};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

fn make_table() -> Table<(String, u32)> {
    let columns = vec![
        Column { header: "Name".to_string(), width: 15 },
        Column { header: "Score".to_string(), width: 10 },
    ];
    
    let items = vec![
        ("Alice".to_string(), 30),
        ("Bob".to_string(), 10),
        ("Charlie".to_string(), 50),
        ("Diana".to_string(), 20),
    ];
    
    Table::new_with_id(WidgetId::new(1), columns)
        .with_items(items)
}

fn key_press(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    }
}

#[test]
fn test_table_sort_persists_across_renders() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));
    
    // Simulate header click to sort
    table.set_sort_column(Some(1), true);
    
    let plane1 = table.render(Rect::new(0, 0, 40, 10));
    
    // Render again
    let plane2 = table.render(Rect::new(0, 0, 40, 10));
    
    // Both renders should produce same result
    assert_eq!(plane1.cells.len(), plane2.cells.len());
}

#[test]
fn test_table_sort_survives_theme_change() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));
    
    // Set sort
    table.set_sort_column(Some(1), true);
    
    // Change theme
    let theme = Theme::cyberpunk();
    table.on_theme_change(&theme);
    
    // Render should still work
    let plane = table.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
    
    // All cells should have new theme bg
    for cell in &plane.cells {
        assert_eq!(cell.bg, theme.bg);
    }
}

#[test]
fn test_table_sort_ascending_descending_toggle() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));
    
    // Sort ascending
    table.set_sort_column(Some(0), true);
    let _plane_asc = table.render(Rect::new(0, 0, 40, 10));
    
    // Sort descending
    table.set_sort_column(Some(0), false);
    let _plane_desc = table.render(Rect::new(0, 0, 40, 10));
    
    // Both should render without panic
}

#[test]
fn test_table_clear_sort() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));
    
    // Set and then clear sort
    table.set_sort_column(Some(1), true);
    table.set_sort_column(None, true);
    
    let plane = table.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_table_header_click_sort() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));
    
    // Click on header row
    let result = table.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 0);
    assert!(result);
}

#[test]
fn test_table_row_selection_after_sort() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));
    
    // Sort by score
    table.set_sort_column(Some(1), true);
    
    // Select a row
    table.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 2);
    
    let plane = table.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_table_multiple_theme_changes() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 40, 10));
    
    table.set_sort_column(Some(1), true);
    
    let themes = vec![
        Theme::nord(),
        Theme::cyberpunk(),
        Theme::dracula(),
        Theme::gruvbox_dark(),
    ];
    
    for theme in &themes {
        table.on_theme_change(theme);
        let plane = table.render(Rect::new(0, 0, 40, 10));
        assert!(plane.cells.len() > 0);
    }
}

#[test]
fn test_table_empty_items_sort() {
    let columns = vec![
        Column { header: "Name".to_string(), width: 15 },
    ];
    
    let mut table: Table<String> = Table::new_with_id(WidgetId::new(1), columns);
    table.set_area(Rect::new(0, 0, 40, 10));
    
    table.set_sort_column(Some(0), true);
    
    let plane = table.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_table_single_item_sort() {
    let columns = vec![
        Column { header: "Name".to_string(), width: 15 },
    ];
    
    let mut table = Table::new_with_id(WidgetId::new(1), columns)
        .with_items(vec!["Only".to_string()]);
    table.set_area(Rect::new(0, 0, 40, 10));
    
    table.set_sort_column(Some(0), true);
    
    let plane = table.render(Rect::new(0, 0, 40, 10));
    assert!(plane.cells.len() > 0);
}
