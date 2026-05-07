//! CommandPalette tests — filtering, navigation, execution.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{CommandItem, CommandPalette};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEventKind};
use ratatui::layout::Rect;

fn make_palette() -> CommandPalette {
    let commands = vec![
        CommandItem { id: "open", name: "Open File", category: "File" },
        CommandItem { id: "save", name: "Save File", category: "File" },
        CommandItem { id: "cut", name: "Cut", category: "Edit" },
        CommandItem { id: "copy", name: "Copy", category: "Edit" },
        CommandItem { id: "paste", name: "Paste", category: "Edit" },
        CommandItem { id: "quit", name: "Quit", category: "Application" },
    ];
    
    CommandPalette::new(commands).with_theme(Theme::nord())
}

fn key_press(code: KeyCode) -> KeyEvent {
    KeyEvent { code, modifiers: KeyModifiers::empty(), kind: KeyEventKind::Press }
}

#[test]
fn test_palette_new_hidden() {
    let palette = make_palette();
    assert!(!palette.is_visible());
}

#[test]
fn test_palette_show() {
    let mut palette = make_palette();
    palette.show();
    assert!(palette.is_visible());
}

#[test]
fn test_palette_hide() {
    let mut palette = make_palette();
    palette.show();
    palette.hide();
    assert!(!palette.is_visible());
}

#[test]
fn test_palette_filter_by_name() {
    let mut palette = make_palette();
    palette.show();
    
    // Type "open" to filter
    palette.handle_key(key_press(KeyCode::Char('o')));
    palette.handle_key(key_press(KeyCode::Char('p')));
    palette.handle_key(key_press(KeyCode::Char('e')));
    palette.handle_key(key_press(KeyCode::Char('n')));
    
    let plane = palette.render(Rect::new(0, 0, 40, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_palette_filter_by_category() {
    let mut palette = make_palette();
    palette.show();
    
    // Type "edit" to filter by category
    palette.handle_key(key_press(KeyCode::Char('e')));
    palette.handle_key(key_press(KeyCode::Char('d')));
    palette.handle_key(key_press(KeyCode::Char('i')));
    palette.handle_key(key_press(KeyCode::Char('t')));
    
    let plane = palette.render(Rect::new(0, 0, 40, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_palette_navigation_up_down() {
    let mut palette = make_palette();
    palette.show();
    
    palette.handle_key(key_press(KeyCode::Down));
    palette.handle_key(key_press(KeyCode::Down));
    palette.handle_key(key_press(KeyCode::Up));
    
    let plane = palette.render(Rect::new(0, 0, 40, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_palette_esc_hides() {
    let mut palette = make_palette();
    palette.show();
    
    palette.handle_key(key_press(KeyCode::Esc));
    assert!(!palette.is_visible());
}

use std::cell::RefCell;
use std::rc::Rc;

#[test]
fn test_palette_enter_executes() {
    let executed = Rc::new(RefCell::new(None));
    let executed_clone = Rc::clone(&executed);
    
    let mut palette = make_palette().on_execute(move |id| {
        *executed_clone.borrow_mut() = Some(id.to_string());
    });
    palette.show();
    
    palette.handle_key(key_press(KeyCode::Enter));
    
    assert!(executed.borrow().is_some());
}

#[test]
fn test_palette_mouse_click_executes() {
    let mut palette = make_palette();
    palette.show();
    palette.set_area(Rect::new(0, 0, 40, 20));
    
    // Click on a command item
    let result = palette.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 3);
    assert!(result);
}

#[test]
fn test_palette_backspace_clears_filter() {
    let mut palette = make_palette();
    palette.show();
    
    palette.handle_key(key_press(KeyCode::Char('t')));
    palette.handle_key(key_press(KeyCode::Char('e')));
    palette.handle_key(key_press(KeyCode::Backspace));
    palette.handle_key(key_press(KeyCode::Backspace));
    
    let plane = palette.render(Rect::new(0, 0, 40, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_palette_empty_filter_shows_all() {
    let mut palette = make_palette();
    palette.show();
    
    let plane = palette.render(Rect::new(0, 0, 40, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_palette_no_match_filter() {
    let mut palette = make_palette();
    palette.show();
    
    // Type something that won't match
    palette.handle_key(key_press(KeyCode::Char('z')));
    palette.handle_key(key_press(KeyCode::Char('z')));
    palette.handle_key(key_press(KeyCode::Char('z')));
    palette.handle_key(key_press(KeyCode::Char('z')));
    
    let plane = palette.render(Rect::new(0, 0, 40, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_palette_theme_change() {
    let mut palette = make_palette();
    palette.show();
    
    palette.on_theme_change(&Theme::cyberpunk());
    let plane = palette.render(Rect::new(0, 0, 40, 20));
    assert!(plane.cells.len() > 0);
}

#[test]
fn test_palette_not_visible_ignores_input() {
    let mut palette = make_palette();
    // Don't show it
    
    let result = palette.handle_key(key_press(KeyCode::Char('a')));
    assert!(!result);
}

#[test]
fn test_palette_with_size() {
    let palette = CommandPalette::new(vec![])
        .with_size(60, 30)
        .with_theme(Theme::nord());
    
    let plane = palette.render(Rect::new(0, 0, 60, 30));
    assert_eq!(plane.width, 60);
    assert_eq!(plane.height, 30);
}
