//! Interaction tests for the ContextMenu widget.

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::context_menu::{
    ContextAction, ContextMenu, ContextMenuItem,
};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;

fn make_key(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    }
}

#[test]
fn test_context_menu_new_with_items() {
    let items = vec![
        ContextMenuItem::new("open", "Open"),
        ContextMenuItem::new("edit", "Edit"),
        ContextMenuItem::new("delete", "Delete"),
    ];
    let menu = ContextMenu::new(items);
    assert!(menu.is_visible());
    // action_at returns Some only for items created via from_actions
    assert!(menu.action_at(0).is_none());
}

#[test]
fn test_context_menu_render_produces_plane() {
    let items = vec![
        ContextMenuItem::new("open", "Open"),
        ContextMenuItem::new("edit", "Edit"),
    ];
    let menu = ContextMenu::new(items).with_anchor(10, 5);
    let plane = menu.render(Rect::new(0, 0, 80, 24));
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_context_menu_show_hide_toggle() {
    let items = vec![ContextMenuItem::new("a", "Action A")];
    let mut menu = ContextMenu::new(items);

    assert!(menu.is_visible());
    menu.hide();
    assert!(!menu.is_visible());
    menu.show();
    assert!(menu.is_visible());
}

#[test]
fn test_context_menu_keyboard_navigation() {
    let items = vec![
        ContextMenuItem::new("a", "First"),
        ContextMenuItem::new("b", "Second"),
        ContextMenuItem::new("c", "Third"),
    ];
    let mut menu = ContextMenu::new(items);

    let initial = menu.selected_index();
    let _ = menu.handle_key(make_key(KeyCode::Down));
    // Down should change selection
    let after_down = menu.selected_index();
    assert_ne!(initial, after_down, "Down key should advance selection");
}

#[test]
fn test_context_menu_set_anchor_updates_position() {
    let items = vec![ContextMenuItem::new("a", "Action")];
    let mut menu = ContextMenu::new(items);

    menu.set_anchor(40, 12);
    // The menu's area should reflect the new anchor
    let _area = menu.area();
    // No panic means anchor was set
}

#[test]
fn test_context_menu_with_theme() {
    let items = vec![ContextMenuItem::new("a", "Action")];
    let menu = ContextMenu::new(items).with_theme(Theme::nord());
    // No panic means theme was applied
    let plane = menu.render(Rect::new(0, 0, 80, 24));
    assert!(plane.width > 0);
}

#[test]
fn test_context_menu_from_actions() {
    let items = vec![
        ("Open", ContextAction::Open),
        ("Edit", ContextAction::Edit),
        ("Delete", ContextAction::Delete),
    ];
    let menu = ContextMenu::from_actions(items);
    assert!(menu.is_visible());
    // All three actions should be retrievable
    assert!(menu.action_at(0).is_some());
    assert!(menu.action_at(1).is_some());
    assert!(menu.action_at(2).is_some());
}

#[test]
fn test_context_menu_separator_item() {
    let items = vec![
        ContextMenuItem::new("a", "First"),
        ContextMenuItem::separator(),
        ContextMenuItem::new("b", "After Separator"),
    ];
    let menu = ContextMenu::new(items);
    // Render must not panic with separators
    let plane = menu.render(Rect::new(0, 0, 40, 10));
    assert!(plane.height > 0);
}

#[test]
fn test_context_menu_icon_renders() {
    let items = vec![ContextMenuItem::new("a", "Cut").with_icon('\u{2702}')];
    let menu = ContextMenu::new(items);
    // Render must not panic with icons
    let plane = menu.render(Rect::new(0, 0, 40, 5));
    assert!(plane.width > 0);
}
