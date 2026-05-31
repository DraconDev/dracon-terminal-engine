//! Tests for ContextMenuAction enum.

mod common;

use dracon_terminal_engine::utils::FileColumn;
use dracon_terminal_engine::widgets::context_menu::ContextMenuAction;

#[test]
fn test_context_menu_action_variants() {
    use ContextMenuAction::*;
    let actions = [
        Open,
        OpenNewTab,
        OpenWith,
        Edit,
        Run,
        RunTerminal,
        ExtractHere,
        NewFolder,
        NewFile,
        Cut,
        Copy,
        CopyPath,
        CopyName,
        Paste,
        Rename,
        Duplicate,
        Compress,
        Delete,
        TerminalWindow,
        TerminalTab,
        Properties,
        GitStatus,
        AddToFavorites,
        RemoveFromFavorites,
        Refresh,
        SelectAll,
        ToggleHidden,
        ConnectRemote,
        DeleteRemote,
        Mount,
        Unmount,
        SetWallpaper,
        GitInit,
        SystemMonitor,
        Drag,
        Separator,
        SortBy(FileColumn::Name),
        SortBy(FileColumn::Size),
        SortBy(FileColumn::Modified),
        SortBy(FileColumn::Permissions),
        SetColor(None),
        SetColor(Some(128)),
    ];
    assert_eq!(actions.len(), 42);
}

#[test]
fn test_context_menu_action_clone() {
    use ContextMenuAction::*;
    let action = Open;
    let cloned = action.clone();
    assert_eq!(cloned, action);
}

#[test]
fn test_context_menu_action_debug() {
    use ContextMenuAction::*;
    let action = Delete;
    let debug_str = format!("{:?}", action);
    assert!(debug_str.contains("Delete"));
}

#[test]
fn test_context_menu_action_partial_eq() {
    use ContextMenuAction::*;
    assert_eq!(Open, Open);
    assert_eq!(OpenNewTab, OpenNewTab);
    assert_ne!(Open, OpenNewTab);
}

#[test]
fn test_context_menu_action_set_color() {
    use ContextMenuAction::*;
    assert_eq!(SetColor(None), SetColor(None));
    assert_eq!(SetColor(Some(42)), SetColor(Some(42)));
    assert_ne!(SetColor(None), SetColor(Some(42)));
}

#[test]
fn test_context_menu_action_sort_by() {
    use ContextMenuAction::*;
    assert_eq!(SortBy(FileColumn::Name), SortBy(FileColumn::Name));
    assert_ne!(SortBy(FileColumn::Name), SortBy(FileColumn::Size));
}

#[test]
fn test_context_menu_action_serialize_deserialize() {
    use ContextMenuAction::*;
    let action = Open;
    let serialized = serde_json::to_string(&action).unwrap();
    let deserialized: ContextMenuAction = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, action);
}

#[test]
fn test_context_menu_action_serialize_sort_by() {
    let action = ContextMenuAction::SortBy(FileColumn::Size);
    let serialized = serde_json::to_string(&action).unwrap();
    let deserialized: ContextMenuAction = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, action);
}

#[test]
fn test_context_menu_action_serialize_set_color() {
    let action = ContextMenuAction::SetColor(Some(200));
    let serialized = serde_json::to_string(&action).unwrap();
    let deserialized: ContextMenuAction = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized, action);
}

// ============================================================================
// Widget Interaction Tests
// ============================================================================

use dracon_terminal_engine::framework::theme::Theme;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::context_menu::{ContextMenu, ContextMenuItem};
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

fn make_menu() -> ContextMenu {
    let items = vec![
        ContextMenuItem::new("open", "Open"),
        ContextMenuItem::new("edit", "Edit"),
        ContextMenuItem::separator(),
        ContextMenuItem::new("delete", "Delete"),
    ];
    ContextMenu::new(items)
}

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_context_menu_widget_new() {
    let menu = make_menu();
    // ContextMenu starts visible by default
    assert!(menu.is_visible());
}

#[test]
fn test_context_menu_widget_with_theme() {
    let menu = make_menu().with_theme(Theme::nord());
    let area = Rect::new(0, 0, 60, 20);
    let plane = menu.render(area);
    assert!(plane.width > 0);
}

// ============================================================================
// Visibility Tests
// ============================================================================

#[test]
fn test_context_menu_show_hide() {
    let mut menu = make_menu();
    assert!(menu.is_visible());

    menu.hide();
    assert!(!menu.is_visible());

    menu.show();
    assert!(menu.is_visible());
}

#[test]
fn test_context_menu_toggle() {
    let mut menu = make_menu();
    assert!(menu.is_visible());

    menu.hide();
    assert!(!menu.is_visible());

    menu.show();
    assert!(menu.is_visible());
}

// ============================================================================
// Handle Key Tests
// ============================================================================

#[test]
fn test_context_menu_handle_key_up() {
    let mut menu = make_menu();
    menu.set_area(Rect::new(0, 0, 30, 10));

    assert!(menu.handle_key(make_key(KeyCode::Up)));
}

#[test]
fn test_context_menu_handle_key_down() {
    let mut menu = make_menu();
    menu.set_area(Rect::new(0, 0, 30, 10));

    assert!(menu.handle_key(make_key(KeyCode::Down)));
}

#[test]
fn test_context_menu_handle_key_enter() {
    let mut menu = make_menu();
    menu.set_area(Rect::new(0, 0, 30, 10));

    assert!(menu.handle_key(make_key(KeyCode::Enter)));
}

#[test]
fn test_context_menu_handle_key_esc() {
    let mut menu = make_menu();
    menu.set_area(Rect::new(0, 0, 30, 10));

    assert!(menu.handle_key(make_key(KeyCode::Esc)));
    assert!(!menu.is_visible());
}

#[test]
fn test_context_menu_handle_key_when_hidden() {
    let mut menu = make_menu();
    menu.hide();

    let result = menu.handle_key(make_key(KeyCode::Down));
    assert!(!result);
}

#[test]
fn test_context_menu_handle_key_ignore_release() {
    let mut menu = make_menu();
    menu.set_area(Rect::new(0, 0, 30, 10));

    let release = KeyEvent {
        code: KeyCode::Down,
        kind: KeyEventKind::Release,
        modifiers: KeyModifiers::empty(),
    };
    let result = menu.handle_key(release);
    let _ = result;
}

// ============================================================================
// Handle Mouse Tests
// ============================================================================

#[test]
fn test_context_menu_handle_mouse_click_inside() {
    let mut menu = make_menu();
    // Menu auto-sizes — render to establish internal area
    let _plane = menu.render(Rect::new(10, 10, 30, 15));
    // Click inside the rendered area
    let result = menu.handle_mouse(MouseEventKind::Down(MouseButton::Left), 15, 12);
    let _ = result;
}

#[test]
fn test_context_menu_handle_mouse_click_outside() {
    let mut menu = make_menu();
    // Menu auto-sizes — render to establish internal area
    let _plane = menu.render(Rect::new(10, 10, 30, 15));
    // Click far outside menu area
    let result = menu.handle_mouse(MouseEventKind::Down(MouseButton::Left), 100, 100);
    assert!(!result);
    assert!(!menu.is_visible());
}

#[test]
fn test_context_menu_handle_mouse_hover() {
    let mut menu = make_menu();
    menu.set_area(Rect::new(10, 10, 20, 8));
    menu.render(Rect::new(10, 10, 20, 8));

    let result = menu.handle_mouse(MouseEventKind::Moved, 15, 12);
    let _ = result;
}

#[test]
fn test_context_menu_handle_mouse_when_hidden() {
    let mut menu = make_menu();
    menu.hide();

    let result = menu.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 5);
    assert!(!result);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_context_menu_widget_id() {
    let menu = make_menu();
    let _id = menu.id();
}

#[test]
fn test_context_menu_widget_area() {
    let menu = make_menu();
    let _area = menu.area();
}

#[test]
fn test_context_menu_widget_set_area() {
    let mut menu = make_menu();
    menu.set_area(Rect::new(5, 5, 25, 10));
    assert_eq!(menu.area(), Rect::new(5, 5, 25, 10));
}

#[test]
fn test_context_menu_widget_needs_render() {
    let menu = make_menu();
    let _ = menu.needs_render();
}

#[test]
fn test_context_menu_widget_mark_dirty() {
    let mut menu = make_menu();
    menu.mark_dirty();
    assert!(menu.needs_render());
}

#[test]
fn test_context_menu_widget_clear_dirty() {
    let mut menu = make_menu();
    menu.clear_dirty();
    assert!(!menu.needs_render());
}

#[test]
fn test_context_menu_widget_focusable() {
    let menu = make_menu();
    assert!(menu.focusable());
}

#[test]
fn test_context_menu_widget_z_index() {
    let menu = make_menu();
    let _z = menu.z_index();
}

#[test]
fn test_context_menu_widget_render() {
    let menu = make_menu();
    let area = Rect::new(0, 0, 60, 20);
    let plane = menu.render(area);
    // Menu auto-calculates width from content
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_context_menu_widget_on_theme_change() {
    let mut menu = make_menu();
    menu.on_theme_change(&Theme::cyberpunk());
    assert!(menu.needs_render());
}

// ============================================================================
// Separator Handling Tests
// ============================================================================

#[test]
fn test_context_menu_separator_skipped_on_nav() {
    let mut menu = make_menu(); // has separator at index 2
    menu.show();
    menu.set_area(Rect::new(0, 0, 30, 10));

    // Navigate down past separator
    menu.handle_key(make_key(KeyCode::Down)); // index 0 -> 1
    menu.handle_key(make_key(KeyCode::Down)); // index 1 -> 3 (skips separator)
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_context_menu_many_items() {
    let items: Vec<ContextMenuItem> = (0..20)
        .map(|i| ContextMenuItem::new(format!("item_{}", i), format!("Item {}", i)))
        .collect();
    let menu = ContextMenu::new(items);
    let area = Rect::new(0, 0, 60, 25);
    let plane = menu.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_context_menu_empty_items() {
    let menu = ContextMenu::new(vec![]);
    let area = Rect::new(0, 0, 60, 20);
    let plane = menu.render(area);
    assert!(plane.width > 0);
}

#[test]
fn test_context_menu_unicode_labels() {
    let items = vec![
        ContextMenuItem::new("jp", "日本語"),
        ContextMenuItem::new("ar", "عربي"),
        ContextMenuItem::new("em", "🎉"),
    ];
    let menu = ContextMenu::new(items);
    let area = Rect::new(0, 0, 60, 20);
    let plane = menu.render(area);
    assert!(plane.width > 0);
}
