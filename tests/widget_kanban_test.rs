//! Tests for the Kanban board widget.

use dracon_terminal_engine::compositor::Color;
use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::kanban::{Kanban, KanbanCard};
use dracon_terminal_engine::input::event::{
    KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind,
};
use ratatui::layout::Rect;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    }
}

// ============================================================================
// KanbanCard Tests
// ============================================================================

#[test]
fn test_kanban_card_new() {
    let card = KanbanCard::new("id1", "Test Card");

    assert_eq!(card.id, "id1");
    assert_eq!(card.title, "Test Card");
    assert!(card.description.is_none());
    assert!(card.color.is_none());
}

#[test]
fn test_kanban_card_with_description() {
    let card = KanbanCard::new("id1", "Test").with_description("A description");

    assert_eq!(card.title, "Test");
    assert_eq!(card.description, Some("A description".to_string()));
}

#[test]
fn test_kanban_card_with_color() {
    let card = KanbanCard::new("id1", "Test").with_color(Color::Rgb(255, 0, 0));

    assert_eq!(card.color, Some(Color::Rgb(255, 0, 0)));
}

#[test]
fn test_kanban_card_chained_builders() {
    let card = KanbanCard::new("id1", "Test")
        .with_description("Desc")
        .with_color(Color::Rgb(0, 0, 255));

    assert_eq!(card.title, "Test");
    assert!(card.description.is_some());
    assert!(card.color.is_some());
}

#[test]
fn test_kanban_card_clone() {
    let card = KanbanCard::new("id1", "Test");
    let cloned = card.clone();

    assert_eq!(card.id, cloned.id);
    assert_eq!(card.title, cloned.title);
}

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_kanban_new_empty() {
    let board = Kanban::new();

    assert_eq!(board.column_count(), 0);
}

#[test]
fn test_kanban_default() {
    let board = Kanban::default();
    assert_eq!(board.column_count(), 0);
}

#[test]
fn test_kanban_with_columns() {
    let board = Kanban::with_columns(vec![
        ("To Do", vec!["Task 1", "Task 2"]),
        ("Done", vec!["Task 3"]),
    ]);

    assert_eq!(board.column_count(), 2);
    assert_eq!(board.card_count(0), Some(2));
    assert_eq!(board.card_count(1), Some(1));
}

#[test]
fn test_kanban_with_columns_empty() {
    let board = Kanban::with_columns(vec![]);

    assert_eq!(board.column_count(), 0);
}

#[test]
fn test_kanban_with_columns_empty_cards() {
    let board = Kanban::with_columns(vec![("To Do", vec![]), ("Done", vec![])]);

    assert_eq!(board.column_count(), 2);
    assert_eq!(board.card_count(0), Some(0));
    assert_eq!(board.card_count(1), Some(0));
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_kanban_with_theme() {
    let board = Kanban::new().with_theme(Theme::nord());
    let area = Rect::new(0, 0, 80, 20);
    let plane = board.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 20);
}

#[test]
fn test_kanban_with_column_width() {
    let board = Kanban::new().with_column_width(30);
    let area = Rect::new(0, 0, 80, 20);
    let _plane = board.render(area);
}

#[test]
fn test_kanban_with_card_height() {
    let board = Kanban::new().with_card_height(6);
    let area = Rect::new(0, 0, 80, 20);
    let _plane = board.render(area);
}

#[test]
fn test_kanban_on_card_move() {
    let _board = Kanban::new().on_card_move(|_id, _from, _to| {});
}

#[test]
fn test_kanban_chained_builders() {
    let board = Kanban::new()
        .with_theme(Theme::cyberpunk())
        .with_column_width(25)
        .with_card_height(5)
        .on_card_move(|_id, _from, _to| {});

    let area = Rect::new(0, 0, 80, 20);
    let _plane = board.render(area);
}

// ============================================================================
// Column Management Tests
// ============================================================================

#[test]
fn test_kanban_add_column() {
    let mut board = Kanban::new();

    board.add_column("To Do");
    assert_eq!(board.column_count(), 1);

    board.add_column("In Progress");
    assert_eq!(board.column_count(), 2);

    board.add_column("Done");
    assert_eq!(board.column_count(), 3);
}

#[test]
fn test_kanban_add_column_empty_title() {
    let mut board = Kanban::new();
    board.add_column("");
    assert_eq!(board.column_count(), 1);
}

#[test]
fn test_kanban_add_card() {
    let mut board = Kanban::new();
    board.add_column("To Do");

    let card = KanbanCard::new("1", "Task 1");
    board.add_card(0, card);

    assert_eq!(board.card_count(0), Some(1));
}

#[test]
fn test_kanban_add_card_invalid_column() {
    let mut board = Kanban::new();

    let card = KanbanCard::new("1", "Task 1");
    board.add_card(0, card); // No columns yet

    // Should not panic, card should not be added
    assert!(board.card_count(0).is_none());
}

#[test]
fn test_kanban_add_multiple_cards() {
    let mut board = Kanban::new();
    board.add_column("To Do");

    for i in 0..5 {
        board.add_card(0, KanbanCard::new(i.to_string(), format!("Task {}", i)));
    }

    assert_eq!(board.card_count(0), Some(5));
}

// ============================================================================
// Card Selection Tests
// ============================================================================

#[test]
fn test_kanban_selected_card_none_initially() {
    let board = Kanban::new();
    assert!(board.selected_card().is_none());
}

#[test]
fn test_kanban_select_card() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1", "Task 2"])]);

    board.select_card(0, 0);
    assert_eq!(board.selected_card(), Some((0, 0)));
}

#[test]
fn test_kanban_select_card_out_of_bounds() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    board.select_card(0, 5); // Out of bounds
    assert!(board.selected_card().is_none());

    board.select_card(5, 0); // Column out of bounds
    assert!(board.selected_card().is_none());
}

#[test]
fn test_kanban_clear_selection() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    board.select_card(0, 0);
    assert!(board.selected_card().is_some());

    board.clear_selection();
    assert!(board.selected_card().is_none());
}

#[test]
fn test_kanban_select_card_updates_selection() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1", "Task 2", "Task 3"])]);

    board.select_card(0, 0);
    assert_eq!(board.selected_card(), Some((0, 0)));

    board.select_card(0, 2);
    assert_eq!(board.selected_card(), Some((0, 2)));
}

// ============================================================================
// Card Movement Tests
// ============================================================================

#[test]
fn test_kanban_move_card_same_column() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1", "Task 2", "Task 3"])]);

    board.move_card(0, 0, 0, 2);

    // Card should be at new position
    assert_eq!(board.card_count(0), Some(3));
}

#[test]
fn test_kanban_move_card_between_columns() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"]), ("Done", vec!["Task 2"])]);

    board.move_card(0, 0, 1, 0);

    assert_eq!(board.card_count(0), Some(0));
    assert_eq!(board.card_count(1), Some(2)); // Original + moved
}

#[test]
fn test_kanban_move_card_invalid_from_column() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    board.move_card(5, 0, 0, 0); // Invalid from column
    assert_eq!(board.card_count(0), Some(1)); // Unchanged
}

#[test]
fn test_kanban_move_card_invalid_to_column() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    board.move_card(0, 0, 5, 0); // Invalid to column
    assert_eq!(board.card_count(0), Some(1)); // Unchanged
}

#[test]
fn test_kanban_move_card_invalid_index() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    board.move_card(0, 5, 0, 0); // Invalid index
    assert_eq!(board.card_count(0), Some(1)); // Unchanged
}

// ============================================================================
// Card Removal Tests
// ============================================================================

#[test]
fn test_kanban_remove_card() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1", "Task 2"])]);

    let removed = board.remove_card(0, 0);

    assert!(removed.is_some());
    assert_eq!(removed.unwrap().title, "Task 1");
    assert_eq!(board.card_count(0), Some(1));
}

#[test]
fn test_kanban_remove_card_invalid_column() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    let removed = board.remove_card(5, 0);
    assert!(removed.is_none());
}

#[test]
fn test_kanban_remove_card_invalid_index() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    let removed = board.remove_card(0, 5);
    assert!(removed.is_none());
}

#[test]
fn test_kanban_remove_all_cards() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1", "Task 2", "Task 3"])]);

    board.remove_card(0, 0);
    board.remove_card(0, 0);
    board.remove_card(0, 0);

    assert_eq!(board.card_count(0), Some(0));
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_kanban_id() {
    let board = Kanban::new();
    let _id = board.id();
}

#[test]
fn test_kanban_area() {
    let board = Kanban::new();
    let area = board.area();
    assert_eq!(area.width, 80);
    assert_eq!(area.height, 20);
}

#[test]
fn test_kanban_set_area() {
    let mut board = Kanban::new();
    let new_area = Rect::new(10, 20, 100, 30);
    board.set_area(new_area);
    assert_eq!(board.area(), new_area);
}

#[test]
fn test_kanban_needs_render() {
    let board = Kanban::new();
    assert!(board.needs_render());
}

#[test]
fn test_kanban_mark_dirty() {
    let mut board = Kanban::new();
    board.clear_dirty();
    assert!(!board.needs_render());
    board.mark_dirty();
    assert!(board.needs_render());
}

#[test]
fn test_kanban_clear_dirty() {
    let mut board = Kanban::new();
    board.clear_dirty();
    assert!(!board.needs_render());
}

#[test]
fn test_kanban_render() {
    let board = Kanban::new();
    let area = Rect::new(0, 0, 80, 20);
    let plane = board.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 20);
}

#[test]
fn test_kanban_render_with_columns() {
    let board = Kanban::with_columns(vec![("To Do", vec!["Task 1"]), ("Done", vec!["Task 2"])]);
    let area = Rect::new(0, 0, 80, 20);
    let plane = board.render(area);
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 20);
}

#[test]
fn test_kanban_render_minimal_area() {
    let board = Kanban::new();
    let area = Rect::new(0, 0, 10, 5);
    let plane = board.render(area);
    assert_eq!(plane.width, 10);
    assert_eq!(plane.height, 5);
}

#[test]
fn test_kanban_focusable() {
    let board = Kanban::new();
    assert!(board.focusable());
}

#[test]
fn test_kanban_z_index() {
    let board = Kanban::new();
    assert_eq!(board.z_index(), 10);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_kanban_different_themes() {
    for theme_name in ["nord", "dracula", "monokai", "solarized_dark"] {
        if let Some(theme) = Theme::from_name(theme_name) {
            let board =
                Kanban::with_columns(vec![("To Do", vec!["Task 1", "Task 2"])]).with_theme(theme);

            let area = Rect::new(0, 0, 80, 20);
            let plane = board.render(area);
            assert_eq!(plane.width, 80);
        }
    }
}

// ============================================================================
// Handle Key Tests
// ============================================================================

#[test]
fn test_kanban_handle_key_arrow_keys() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1", "Task 2"])]);

    let _ = board.handle_key(make_key(KeyCode::Left));
    let _ = board.handle_key(make_key(KeyCode::Right));
    let _ = board.handle_key(make_key(KeyCode::Up));
    let _ = board.handle_key(make_key(KeyCode::Down));
}

#[test]
fn test_kanban_handle_key_enter() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    let _ = board.handle_key(make_key(KeyCode::Enter));
}

#[test]
fn test_kanban_handle_key_escape() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    let _ = board.handle_key(make_key(KeyCode::Esc));
}

#[test]
fn test_kanban_handle_key_delete() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    let _ = board.handle_key(make_key(KeyCode::Delete));
}

#[test]
fn test_kanban_handle_key_character() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    let _ = board.handle_key(make_key(KeyCode::Char('a')));
}

// ============================================================================
// Handle Mouse Tests
// ============================================================================

#[test]
fn test_kanban_handle_mouse() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    let area = Rect::new(0, 0, 80, 20);
    board.render(area);

    let _ = board.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 5);
}

#[test]
fn test_kanban_handle_mouse_outside() {
    let mut board = Kanban::with_columns(vec![("To Do", vec!["Task 1"])]);

    let area = Rect::new(0, 0, 80, 20);
    board.render(area);

    // Click outside board area
    let _ = board.handle_mouse(MouseEventKind::Down(MouseButton::Left), 100, 100);
}

#[test]
fn test_kanban_handle_mouse_middle() {
    let mut board = Kanban::new();

    let area = Rect::new(0, 0, 80, 20);
    board.render(area);

    let _ = board.handle_mouse(MouseEventKind::Down(MouseButton::Middle), 5, 5);
}

// ============================================================================
// Drag Manager Tests
// ============================================================================

#[test]
fn test_kanban_drag_manager_access() {
    let board = Kanban::new();
    let _dm = board.drag_manager();
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_kanban_many_columns() {
    let mut board = Kanban::new();

    for i in 0..10 {
        board.add_column(&format!("Column {}", i));
    }

    assert_eq!(board.column_count(), 10);
}

#[test]
fn test_kanban_many_cards_per_column() {
    let mut board = Kanban::new();
    board.add_column("To Do");

    for i in 0..50 {
        board.add_card(0, KanbanCard::new(i.to_string(), format!("Task {}", i)));
    }

    assert_eq!(board.card_count(0), Some(50));
}

#[test]
fn test_kanban_unicode_in_titles() {
    let board = Kanban::with_columns(vec![
        ("日本語", vec!["タスク1", "タスク2"]),
        ("Emoji", vec!["🎉", "🎊"]),
    ]);

    assert_eq!(board.column_count(), 2);
    assert_eq!(board.card_count(0), Some(2));
}

#[test]
fn test_kanban_long_titles() {
    let long_title = "a".repeat(1000);
    let card = KanbanCard::new("id", long_title);

    assert_eq!(card.title.len(), 1000);
}

#[test]
fn test_kanban_empty_columns_and_cards() {
    let board = Kanban::new();
    let area = Rect::new(0, 0, 80, 20);
    let plane = board.render(area);

    // Should render without issue
    assert_eq!(plane.width, 80);
    assert_eq!(plane.height, 20);
}

// ============================================================================
// Rendering Tests
// ============================================================================

#[test]
fn test_kanban_render_fills_bg() {
    let board = Kanban::new();
    let area = Rect::new(0, 0, 80, 20);
    let plane = board.render(area);

    let theme = Theme::default();
    assert_eq!(plane.cells[0].bg, theme.bg);
}

#[test]
fn test_kanban_render_has_content_with_columns() {
    let board = Kanban::with_columns(vec![("To Do", vec!["Task 1"]), ("Done", vec!["Task 2"])]);
    let area = Rect::new(0, 0, 80, 20);
    let plane = board.render(area);

    // Should have some content rendered
    let has_content = plane.cells.iter().any(|c| c.char != '\0' && c.char != ' ');
    assert!(has_content, "Kanban should render some content");
}

// ============================================================================
// Card Move Callback Tests
// ============================================================================

#[test]
fn test_kanban_card_move_callback_registration() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let moves = Rc::new(RefCell::new(Vec::new()));
    let moves_clone = Rc::clone(&moves);

    let _board = Kanban::with_columns(vec![("To Do", vec!["Task 1"]), ("Done", vec!["Task 2"])])
        .on_card_move(move |id, from, to| {
            moves_clone.borrow_mut().push((id, from, to));
        });
}

// ============================================================================
// Integration Tests
// ============================================================================

#[test]
fn test_kanban_full_workflow() {
    let mut board = Kanban::new();

    // Add columns
    board.add_column("To Do");
    board.add_column("In Progress");
    board.add_column("Done");

    // Add cards
    board.add_card(0, KanbanCard::new("1", "Task 1"));
    board.add_card(0, KanbanCard::new("2", "Task 2"));
    board.add_card(0, KanbanCard::new("3", "Task 3"));

    assert_eq!(board.column_count(), 3);
    assert_eq!(board.card_count(0), Some(3));

    // Select and move card
    board.select_card(0, 0);
    board.move_card(0, 0, 1, 0);

    assert_eq!(board.card_count(0), Some(2));
    assert_eq!(board.card_count(1), Some(1));

    // Verify selection still works
    assert_eq!(board.selected_card(), Some((0, 0)));
}
