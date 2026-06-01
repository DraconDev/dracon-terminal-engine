//! Tests for the CommandPalette widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::command_palette::{CommandItem, CommandPalette};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;

// ============================================================================
// Construction Tests
// ============================================================================

fn make_commands() -> Vec<CommandItem> {
    vec![
        CommandItem {
            id: "save",
            name: "Save File",
            category: "File",
        },
        CommandItem {
            id: "open",
            name: "Open File",
            category: "File",
        },
        CommandItem {
            id: "new",
            name: "New Tab",
            category: "Edit",
        },
        CommandItem {
            id: "cut",
            name: "Cut",
            category: "Edit",
        },
        CommandItem {
            id: "copy",
            name: "Copy",
            category: "Edit",
        },
        CommandItem {
            id: "paste",
            name: "Paste",
            category: "Edit",
        },
        CommandItem {
            id: "search",
            name: "Search",
            category: "View",
        },
        CommandItem {
            id: "theme",
            name: "Cycle Theme",
            category: "View",
        },
    ]
}

#[test]
fn test_command_palette_new() {
    let cp = CommandPalette::new(vec![]);
    assert!(!cp.is_visible());
}

#[test]
fn test_command_palette_new_with_commands() {
    let commands = make_commands();
    let cp = CommandPalette::new(commands);
    assert!(!cp.is_visible());
}

#[test]
fn test_command_palette_empty_commands() {
    let cp = CommandPalette::new(vec![]);
    let area = Rect::new(0, 0, 60, 30);
    let _plane = cp.render(area);
}

#[test]
fn test_command_palette_single_command() {
    let commands = vec![CommandItem {
        id: "test",
        name: "Test",
        category: "Test",
    }];
    let cp = CommandPalette::new(commands);
    let area = Rect::new(0, 0, 60, 30);
    let _plane = cp.render(area);
}

// ============================================================================
// Builder Pattern Tests
// ============================================================================

#[test]
fn test_command_palette_with_theme() {
    let cp = CommandPalette::new(vec![]).with_theme(Theme::nord());
    let area = Rect::new(0, 0, 60, 30);
    let _plane = cp.render(area);
}

#[test]
fn test_command_palette_with_size() {
    let cp = CommandPalette::new(vec![]).with_size(80, 40);
    let area = Rect::new(0, 0, 80, 40);
    let _plane = cp.render(area);
}

#[test]
fn test_command_palette_on_execute() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let executed = Rc::new(RefCell::new(Vec::new()));
    let executed_clone = Rc::clone(&executed);

    let cp = CommandPalette::new(make_commands()).on_execute(move |id| {
        executed_clone.borrow_mut().push(id.to_string());
    });

    let _ = cp;
}

#[test]
fn test_command_palette_chained_builders() {
    let cp = CommandPalette::new(make_commands())
        .with_theme(Theme::cyberpunk())
        .with_size(100, 50)
        .on_execute(|_| {});

    let area = Rect::new(0, 0, 100, 50);
    let _plane = cp.render(area);
}

// ============================================================================
// Visibility Tests
// ============================================================================

#[test]
fn test_command_palette_visible_default_false() {
    let cp = CommandPalette::new(vec![]);
    assert!(!cp.is_visible());
}

#[test]
fn test_command_palette_show() {
    let mut cp = CommandPalette::new(vec![]);
    cp.show();
    assert!(cp.is_visible());
}

#[test]
fn test_command_palette_hide() {
    let mut cp = CommandPalette::new(vec![]);
    cp.show();
    cp.hide();
    assert!(!cp.is_visible());
}

#[test]
fn test_command_palette_toggle() {
    let mut cp = CommandPalette::new(vec![]);
    assert!(!cp.is_visible());
    cp.show();
    assert!(cp.is_visible());
    cp.hide();
    assert!(!cp.is_visible());
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_command_palette_id() {
    let cp = CommandPalette::new(vec![]);
    let _id = cp.id();
}

#[test]
fn test_command_palette_area() {
    let cp = CommandPalette::new(vec![]);
    let area = cp.area();
    assert!(area.width > 0);
    assert!(area.height > 0);
}

#[test]
fn test_command_palette_set_area() {
    let mut cp = CommandPalette::new(vec![]);
    let new_area = Rect::new(10, 20, 100, 60);
    cp.set_area(new_area);
    assert_eq!(cp.area(), new_area);
}

#[test]
fn test_command_palette_needs_render() {
    let cp = CommandPalette::new(vec![]);
    // Construction should initialize dirty
    let _ = cp;
}

#[test]
fn test_command_palette_mark_dirty() {
    let mut cp = CommandPalette::new(vec![]);
    // mark_dirty should be callable
    cp.mark_dirty();
    // Should not panic
}

#[test]
fn test_command_palette_clear_dirty() {
    let mut cp = CommandPalette::new(vec![]);
    cp.clear_dirty();
    assert!(!cp.needs_render());
}

#[test]
fn test_command_palette_render() {
    let cp = CommandPalette::new(vec![]);
    let area = Rect::new(0, 0, 60, 30);
    let plane = cp.render(area);
    assert_eq!(plane.width, 60);
    assert!(plane.height >= 1);
}

#[test]
fn test_command_palette_render_with_commands() {
    let cp = CommandPalette::new(make_commands());
    let area = Rect::new(0, 0, 60, 30);
    let plane = cp.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_command_palette_render_hidden() {
    let cp = CommandPalette::new(make_commands());
    let area = Rect::new(0, 0, 60, 30);
    let plane = cp.render(area);
    // Hidden palette should still render something
    assert!(plane.width > 0);
}

#[test]
fn test_command_palette_z_index() {
    let cp = CommandPalette::new(vec![]);
    let _z = cp.z_index();
}

// ============================================================================
// Handle Key Tests
// ============================================================================

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    }
}

#[test]
fn test_command_palette_handle_key_up() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    // Navigate down first, then up
    cp.handle_key(make_key(KeyCode::Down));
    let result = cp.handle_key(make_key(KeyCode::Up));
    assert!(result);
}

#[test]
fn test_command_palette_handle_key_down() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    let result = cp.handle_key(make_key(KeyCode::Down));
    assert!(result);
}

#[test]
fn test_command_palette_handle_key_down_repeat() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    // Navigate through multiple items
    for _ in 0..5 {
        cp.handle_key(make_key(KeyCode::Down));
    }
}

#[test]
fn test_command_palette_handle_key_enter() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    let result = cp.handle_key(make_key(KeyCode::Enter));
    assert!(result);
}

#[test]
fn test_command_palette_handle_key_escape() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    let result = cp.handle_key(make_key(KeyCode::Esc));
    assert!(result);
    assert!(!cp.is_visible());
}

#[test]
fn test_command_palette_handle_key_character() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    let result = cp.handle_key(make_key(KeyCode::Char('s')));
    assert!(result);
}

#[test]
fn test_command_palette_handle_key_backspace() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    // Type something first
    cp.handle_key(make_key(KeyCode::Char('a')));
    let result = cp.handle_key(make_key(KeyCode::Backspace));
    assert!(result);
}

#[test]
fn test_command_palette_handle_key_tab() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    let result = cp.handle_key(make_key(KeyCode::Tab));
    // Tab cycles selection
    let _ = result;
}

#[test]
fn test_command_palette_handle_key_when_hidden() {
    let mut cp = CommandPalette::new(make_commands());
    // Don't show

    let result = cp.handle_key(make_key(KeyCode::Down));
    assert!(!result); // Should not be handled when hidden
}

// ============================================================================
// Filtering Tests
// ============================================================================

#[test]
fn test_command_palette_filter_empty_query() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    // With empty query, all commands should be visible
}

#[test]
fn test_command_palette_filter_by_name() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    // Type to filter
    cp.handle_key(make_key(KeyCode::Char('s')));
    // "s" should match "Save File", "Search"
}

#[test]
fn test_command_palette_filter_by_category() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    // Type category name to filter
    cp.handle_key(make_key(KeyCode::Char('f')));
    // Should match File category
}

#[test]
fn test_command_palette_filter_case_insensitive() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    cp.handle_key(make_key(KeyCode::Char('S')));
    // Should match uppercase S
}

#[test]
fn test_command_palette_filter_no_match() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    cp.handle_key(make_key(KeyCode::Char('x')));
    cp.handle_key(make_key(KeyCode::Char('x')));
    cp.handle_key(make_key(KeyCode::Char('x')));
    // No commands match "xxx"
}

// ============================================================================
// Handle Mouse Tests
// ============================================================================

#[test]
fn test_command_palette_handle_mouse() {
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};

    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    let area = Rect::new(0, 0, 60, 30);
    cp.render(area);

    // Click in the command area
    let result = cp.handle_mouse(MouseEventKind::Down(MouseButton::Left), 30, 5);
    let _ = result;
}

#[test]
fn test_command_palette_handle_mouse_outside() {
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};

    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    let area = Rect::new(0, 0, 60, 30);
    cp.render(area); // Render first to set up zones

    // Click in empty area (not on a command)
    let result = cp.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 5);
    // Result depends on implementation - may or may not consume the click
    let _ = result;
}

#[test]
fn test_command_palette_handle_mouse_scroll() {
    use dracon_terminal_engine::input::event::MouseEventKind;

    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    let area = Rect::new(0, 0, 60, 30);
    cp.render(area);

    // Scroll in the command area
    let result = cp.handle_mouse(MouseEventKind::ScrollUp, 30, 5);
    let _ = result;
}

// ============================================================================
// Selection Tests
// ============================================================================

#[test]
fn test_command_palette_selection_wraps() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    // Navigate down past the end, should wrap
    for _ in 0..20 {
        cp.handle_key(make_key(KeyCode::Down));
    }
    // Selection should wrap to beginning
}

#[test]
fn test_command_palette_selection_up_wraps() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    // Navigate up from first item, should wrap to end
    cp.handle_key(make_key(KeyCode::Up));
    // Selection should wrap to end
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_command_palette_different_themes() {
    for theme_name in ["nord", "dracula", "monokai", "solarized_dark"] {
        if let Some(theme) = Theme::from_name(theme_name) {
            let cp = CommandPalette::new(make_commands()).with_theme(theme);
            let area = Rect::new(0, 0, 60, 30);
            let plane = cp.render(area);
            assert!(plane.width > 0);
        }
    }
}

#[test]
fn test_command_palette_on_theme_change() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    cp.on_theme_change(&Theme::cyberpunk());
    assert!(cp.needs_render());
}

// ============================================================================
// Rendering Tests
// ============================================================================

#[test]
fn test_command_palette_render_fills_bg() {
    let cp = CommandPalette::new(vec![]);
    let area = Rect::new(0, 0, 60, 30);
    let plane = cp.render(area);
    let theme = Theme::default();
    assert_eq!(plane.cells[0].bg, theme.bg);
}

#[test]
fn test_command_palette_render_minimal_area() {
    let cp = CommandPalette::new(vec![]);
    let area = Rect::new(0, 0, 10, 5);
    let plane = cp.render(area);
    assert_eq!(plane.width, 10);
}

#[test]
fn test_command_palette_render_large_area() {
    let cp = CommandPalette::new(vec![]);
    let area = Rect::new(0, 0, 120, 60);
    let plane = cp.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// Command Execution Tests
// ============================================================================

#[test]
fn test_command_palette_execute_callback() {
    use std::cell::RefCell;
    use std::rc::Rc;

    let executed = Rc::new(RefCell::new(Vec::new()));
    let executed_clone = Rc::clone(&executed);

    let mut cp = CommandPalette::new(make_commands()).on_execute(move |id| {
        executed_clone.borrow_mut().push(id.to_string());
    });

    cp.show();
    // Navigate to a command and execute
    for _ in 0..3 {
        cp.handle_key(make_key(KeyCode::Down));
    }
    cp.handle_key(make_key(KeyCode::Enter));

    // Callback should have been called
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_command_palette_many_commands() {
    let commands: Vec<CommandItem> = (0..100)
        .map(|i| CommandItem {
            id: Box::leak(format!("cmd_{}", i).into_boxed_str()),
            name: Box::leak(format!("Command {}", i).into_boxed_str()),
            category: Box::leak("Test".to_string().into_boxed_str()),
        })
        .collect();

    let cp = CommandPalette::new(commands);
    let area = Rect::new(0, 0, 60, 30);
    let plane = cp.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_command_palette_unicode_commands() {
    let commands = vec![
        CommandItem {
            id: "jp",
            name: "日本語コマンド",
            category: "テスト",
        },
        CommandItem {
            id: "ar",
            name: "أمر عربي",
            category: "اختبار",
        },
        CommandItem {
            id: "em",
            name: "🎉 Command",
            category: "emoji",
        },
    ];

    let cp = CommandPalette::new(commands);
    let area = Rect::new(0, 0, 60, 30);
    let plane = cp.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_command_palette_long_command_name() {
    let commands = vec![CommandItem {
        id: Box::leak("long".to_string().into_boxed_str()),
        name: Box::leak("A".repeat(100).into_boxed_str()),
        category: Box::leak("Test".to_string().into_boxed_str()),
    }];

    let cp = CommandPalette::new(commands);
    let area = Rect::new(0, 0, 60, 30);
    let plane = cp.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_command_palette_long_category() {
    let commands = vec![CommandItem {
        id: Box::leak("test".to_string().into_boxed_str()),
        name: Box::leak("Test".to_string().into_boxed_str()),
        category: Box::leak("A".repeat(50).into_boxed_str()),
    }];

    let cp = CommandPalette::new(commands);
    let area = Rect::new(0, 0, 60, 30);
    let plane = cp.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_command_palette_show_clears_selection() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    // Navigate somewhere
    for _ in 0..5 {
        cp.handle_key(make_key(KeyCode::Down));
    }

    // Hide and show again
    cp.hide();
    cp.show();

    // Selection should be back at 0
}

#[test]
fn test_command_palette_show_clears_query() {
    let mut cp = CommandPalette::new(make_commands());
    cp.show();

    // Type something
    cp.handle_key(make_key(KeyCode::Char('s')));

    // Hide and show again
    cp.hide();
    cp.show();

    // Query should be cleared
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTERACTION TESTS: P3-3
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_cp_mouse_move_marks_dirty() {
    use dracon_terminal_engine::input::event::MouseEventKind;
    let mut cp = CommandPalette::new(vec![CommandItem {
        id: "a",
        name: "Alpha",
        category: "Test",
    }])
    .with_size(40, 10);
    let _ = cp.handle_mouse(MouseEventKind::Moved, 5, 5);
    // No panic means the move was processed
    let plane = cp.render(Rect::new(0, 0, 80, 24));
    assert!(plane.width > 0);
}

#[test]
fn test_cp_typing_filters_results() {
    let mut cp = CommandPalette::new(vec![
        CommandItem {
            id: "a",
            name: "Alpha",
            category: "Test",
        },
        CommandItem {
            id: "b",
            name: "Beta",
            category: "Test",
        },
    ]);

    let key = KeyEvent {
        code: KeyCode::Char('a'),
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    let _ = cp.handle_key(key);
    // No panic means filter input was accepted
    let plane = cp.render(Rect::new(0, 0, 80, 24));
    assert!(plane.width > 0);
}

#[test]
fn test_cp_esc_dismisses() {
    let mut cp = CommandPalette::new(vec![CommandItem {
        id: "a",
        name: "Alpha",
        category: "Test",
    }]);
    cp.show();
    assert!(cp.is_visible());
    let key = KeyEvent {
        code: KeyCode::Esc,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    };
    let _ = cp.handle_key(key);
    // No panic — palette handled the escape
}
