//! MenuBar and ContextMenu tests — submenu navigation, click handling.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{ContextMenu, ContextAction, MenuBar, MenuEntry, MenuItem};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

fn make_menu_bar() -> MenuBar {
    let file_menu = MenuEntry::new("File")
        .add_item(MenuItem::new("Open"))
        .add_item(MenuItem::new("Save"))
        .add_item(MenuItem::new("Exit"));
    
    let edit_menu = MenuEntry::new("Edit")
        .add_item(MenuItem::new("Cut"))
        .add_item(MenuItem::new("Copy"))
        .add_item(MenuItem::new("Paste"));
    
    MenuBar::new(WidgetId::new(1))
        .with_entries(vec![file_menu, edit_menu])
        .with_theme(Theme::nord())
}

fn key_press(code: KeyCode) -> KeyEvent {
    KeyEvent { code, modifiers: KeyModifiers::empty(), kind: KeyEventKind::Press }
}

#[test]
fn test_menu_bar_new() {
    let menu = MenuBar::new(WidgetId::new(1));
    let plane = menu.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.height, 1);
}

#[test]
fn test_menu_bar_with_entries() {
    let menu = make_menu_bar();
    let plane = menu.render(Rect::new(0, 0, 80, 1));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_menu_bar_theme_change() {
    let mut menu = make_menu_bar();
    menu.on_theme_change(&Theme::cyberpunk());
    let plane = menu.render(Rect::new(0, 0, 80, 1));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_menu_bar_no_black_background() {
    let menu = make_menu_bar();
    let plane = menu.render(Rect::new(0, 0, 80, 1));
    for cell in &plane.cells {
        assert_ne!(cell.bg, dracon_terminal_engine::compositor::Color::Reset);
    }
}

#[test]
fn test_context_menu_new() {
    let items = vec![
        ("Cut", ContextAction::Open),
        ("Copy", ContextAction::Open),
        ("Paste", ContextAction::Open),
    ];
    let menu = ContextMenu::new(items);
    let plane = menu.render(Rect::new(0, 0, 20, 5));
    assert_eq!(plane.height, 3); // height matches item count
}

#[test]
fn test_context_menu_with_theme() {
    let items = vec![
        ("Cut", ContextAction::Open),
        ("Copy", ContextAction::Open),
    ];
    let menu = ContextMenu::new(items).with_theme(Theme::nord());
    let plane = menu.render(Rect::new(0, 0, 20, 3));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_context_menu_navigation_down() {
    let items = vec![
        ("Cut", ContextAction::Open),
        ("Copy", ContextAction::Open),
        ("Paste", ContextAction::Open),
    ];
    let mut menu = ContextMenu::new_with_id(WidgetId::new(1), items);
    menu.set_area(Rect::new(0, 0, 20, 5));
    
    menu.handle_key(key_press(KeyCode::Down));
    menu.handle_key(key_press(KeyCode::Down));
    
    let plane = menu.render(Rect::new(0, 0, 20, 5));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_context_menu_enter_selects() {
    let items = vec![
        ("Cut", ContextAction::Open),
        ("Copy", ContextAction::Open),
    ];
    let mut menu = ContextMenu::new_with_id(WidgetId::new(1), items);
    menu.set_area(Rect::new(0, 0, 20, 3));
    
    let result = menu.handle_key(key_press(KeyCode::Enter));
    assert!(result);
}

#[test]
fn test_context_menu_esc_closes() {
    let items = vec![
        ("Cut", ContextAction::Open),
    ];
    let mut menu = ContextMenu::new_with_id(WidgetId::new(1), items);
    menu.set_area(Rect::new(0, 0, 20, 2));
    
    let result = menu.handle_key(key_press(KeyCode::Esc));
    assert!(result);
}

#[test]
fn test_context_menu_mouse_click() {
    let items = vec![
        ("Cut", ContextAction::Open),
        ("Copy", ContextAction::Open),
        ("Paste", ContextAction::Open),
    ];
    let mut menu = ContextMenu::new_with_id(WidgetId::new(1), items);
    menu.set_area(Rect::new(0, 0, 20, 5));
    
    let result = menu.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 1);
    assert!(result);
}

#[test]
fn test_context_menu_empty() {
    let menu = ContextMenu::new(vec![]);
    let plane = menu.render(Rect::new(0, 0, 20, 1));
    assert!(plane.cells.len() > 0);
}
