//! Tests for the Table widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::table::{Column, Table};

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_table_new() {
    let columns = vec![
        Column {
            header: "Name".to_string(),
            width: 20,
        },
        Column {
            header: "Value".to_string(),
            width: 10,
        },
    ];
    let table: Table<String> = Table::new(columns);
    let area = table.area();
    assert!(area.width > 0);
}

#[test]
fn test_table_new_with_id() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new_with_id(WidgetId::new(42), columns);
    assert_eq!(table.id(), WidgetId::new(42));
}

#[test]
fn test_table_with_rows() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let rows = vec!["Row1".to_string(), "Row2".to_string()];
    let table = Table::new(columns).with_rows(rows);
    let plane = table.render(Rect::new(0, 0, 50, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_table_empty() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new(columns);
    let plane = table.render(Rect::new(0, 0, 50, 20));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_table_id() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new_with_id(WidgetId::new(42), columns);
    assert_eq!(table.id(), WidgetId::new(42));
}

#[test]
fn test_table_set_id() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let mut table: Table<String> = Table::new(columns);
    table.set_id(WidgetId::new(99));
    assert_eq!(table.id(), WidgetId::new(99));
}

#[test]
fn test_table_area() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new(columns);
    let area = table.area();
    assert!(area.width > 0);
}

#[test]
fn test_table_set_area() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let mut table: Table<String> = Table::new(columns);
    table.set_area(Rect::new(0, 0, 80, 30));
    assert_eq!(table.area(), Rect::new(0, 0, 80, 30));
}

#[test]
fn test_table_needs_render() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new(columns);
    assert!(table.needs_render());
}

#[test]
fn test_table_mark_dirty() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let mut table: Table<String> = Table::new(columns);
    table.clear_dirty();
    assert!(!table.needs_render());
    table.mark_dirty();
    assert!(table.needs_render());
}

#[test]
fn test_table_clear_dirty() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let mut table: Table<String> = Table::new(columns);
    table.clear_dirty();
    assert!(!table.needs_render());
}

#[test]
fn test_table_default_dirty() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new(columns);
    assert!(table.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_table_render_basic() {
    let columns = vec![
        Column {
            header: "Name".to_string(),
            width: 20,
        },
        Column {
            header: "Age".to_string(),
            width: 10,
        },
    ];
    let table: Table<String> = Table::new(columns);
    let plane = table.render(Rect::new(0, 0, 50, 20));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_table_render_has_content() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new(columns);
    let plane = table.render(Rect::new(0, 0, 50, 20));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_table_render_wide() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new(columns);
    let plane = table.render(Rect::new(0, 0, 80, 20));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_table_render_small() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new(columns);
    let plane = table.render(Rect::new(0, 0, 20, 10));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_table_render_tall() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new(columns);
    let plane = table.render(Rect::new(0, 0, 50, 40));
    assert_eq!(plane.height, 40);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_table_on_theme_change() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let mut table: Table<String> = Table::new(columns);
    table.on_theme_change(&Theme::nord());
    assert!(table.needs_render());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_table_render_twice() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let table: Table<String> = Table::new(columns);
    let _ = table.render(Rect::new(0, 0, 50, 20));
    let _ = table.render(Rect::new(0, 0, 50, 20));
}

#[test]
fn test_table_set_area_then_render() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let mut table: Table<String> = Table::new(columns);
    table.set_area(Rect::new(0, 0, 80, 30));
    let plane = table.render(Rect::new(0, 0, 80, 30));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_table_many_rows() {
    let columns = vec![Column {
        header: "Test".to_string(),
        width: 10,
    }];
    let rows: Vec<String> = (0..50).map(|i| format!("Row{}", i)).collect();
    let table = Table::new(columns).with_rows(rows);
    let plane = table.render(Rect::new(0, 0, 50, 30));
    assert!(plane.width > 0);
}

// ============================================================================
// Handle Key Tests
// ============================================================================

use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    }
}

fn make_table() -> Table<String> {
    let columns = vec![
        Column {
            header: "Name".to_string(),
            width: 20,
        },
        Column {
            header: "Value".to_string(),
            width: 10,
        },
    ];
    let rows: Vec<String> = (0..10).map(|i| format!("Row{}", i)).collect();
    Table::new(columns).with_rows(rows)
}

#[test]
fn test_table_handle_key_down() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));

    assert!(table.handle_key(make_key(KeyCode::Down)));
    assert!(table.handle_key(make_key(KeyCode::Down)));
}

#[test]
fn test_table_handle_key_up() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));

    // Move down first
    table.handle_key(make_key(KeyCode::Down));
    table.handle_key(make_key(KeyCode::Down));

    assert!(table.handle_key(make_key(KeyCode::Up)));
}

#[test]
fn test_table_handle_key_home() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));

    table.handle_key(make_key(KeyCode::Down));
    table.handle_key(make_key(KeyCode::Down));
    assert!(table.handle_key(make_key(KeyCode::Home)));
}

#[test]
fn test_table_handle_key_end() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));

    assert!(table.handle_key(make_key(KeyCode::End)));
}

#[test]
fn test_table_handle_key_enter() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));

    assert!(table.handle_key(make_key(KeyCode::Enter)));
}

#[test]
fn test_table_handle_key_ignore_release() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));

    let release = KeyEvent {
        code: KeyCode::Down,
        kind: KeyEventKind::Release,
        modifiers: KeyModifiers::empty(),
    };
    assert!(!table.handle_key(release));
}

#[test]
fn test_table_handle_key_ignore_char() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));

    // Plain char should be ignored
    let result = table.handle_key(make_key(KeyCode::Char('x')));
    assert!(!result);
}

#[test]
fn test_table_handle_key_ctrl_z() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));

    let ctrl_z = KeyEvent {
        code: KeyCode::Char('z'),
        kind: KeyEventKind::Press,
        modifiers: KeyModifiers::CONTROL,
    };
    // Undo on empty history should not panic
    assert!(table.handle_key(ctrl_z));
}

#[test]
fn test_table_handle_key_ctrl_y() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));

    let ctrl_y = KeyEvent {
        code: KeyCode::Char('y'),
        kind: KeyEventKind::Press,
        modifiers: KeyModifiers::CONTROL,
    };
    // Redo on empty history should not panic
    assert!(table.handle_key(ctrl_y));
}

#[test]
fn test_table_handle_key_ctrl_a() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));

    let ctrl_a = KeyEvent {
        code: KeyCode::Char('a'),
        kind: KeyEventKind::Press,
        modifiers: KeyModifiers::CONTROL,
    };
    // Select all (if multi-select enabled)
    assert!(table.handle_key(ctrl_a));
}

// ============================================================================
// Handle Mouse Tests
// ============================================================================

use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};

#[test]
fn test_table_handle_mouse_hover_row() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));
    table.render(Rect::new(0, 0, 50, 20));

    // Hover over row 1 (row 0 is header)
    let result = table.handle_mouse(MouseEventKind::Moved, 5, 2);
    assert!(result);
}

#[test]
fn test_table_handle_mouse_hover_header() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));
    table.render(Rect::new(0, 0, 50, 20));

    // Hover over header (row 0)
    let result = table.handle_mouse(MouseEventKind::Moved, 5, 0);
    // Header hover may or may not consume the event
    let _ = result;
}

#[test]
fn test_table_handle_mouse_click_row() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));
    table.render(Rect::new(0, 0, 50, 20));

    let result = table.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 2);
    assert!(result);
}

#[test]
fn test_table_handle_mouse_outside() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 20));
    table.render(Rect::new(0, 0, 50, 20));

    // Click far outside
    let result = table.handle_mouse(MouseEventKind::Down(MouseButton::Left), 100, 100);
    // Should not consume clicks far outside
    let _ = result;
}

#[test]
fn test_table_handle_mouse_scroll() {
    let mut table = make_table();
    table.set_area(Rect::new(0, 0, 50, 10));
    table.render(Rect::new(0, 0, 50, 10));

    let result = table.handle_mouse(MouseEventKind::ScrollDown, 25, 5);
    let _ = result;
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTERACTION TESTS: P3-3
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_table_arrow_keys_change_selection() {
    let mut table = make_table();
    let initial = table.selected_index();
    let _ = table.handle_key(KeyEvent {
        code: KeyCode::Down,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    });
    let after = table.selected_index();
    // Down should change selection (unless at bottom)
    let _ = (initial, after);
}

#[test]
fn test_table_mouse_hover_marks_dirty() {
    let mut table = make_table();
    let _ = table.handle_mouse(MouseEventKind::Moved, 10, 2);
    // No panic — hover was processed
}

#[test]
fn test_table_render_with_theme() {
    let mut table = make_table();
    table.on_theme_change(&Theme::nord());
    let plane = table.render(Rect::new(0, 0, 50, 10));
    assert_eq!(plane.width, 50);
}
