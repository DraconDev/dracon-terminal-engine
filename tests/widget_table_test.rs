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
