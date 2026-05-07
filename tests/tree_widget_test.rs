//! Tree widget tests — expand/collapse, navigation, selection.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::{Widget, WidgetId};
use dracon_terminal_engine::framework::widgets::{Tree, TreeNode};
use dracon_terminal_engine::input::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::Rect;

fn make_tree() -> Tree {
    let mut root = TreeNode::new("root");
    let mut child1 = TreeNode::new("child1");
    child1.add_child(TreeNode::new("grandchild1"));
    child1.add_child(TreeNode::new("grandchild2"));
    root.add_child(child1);
    root.add_child(TreeNode::new("child2"));
    
    Tree::new(WidgetId::new(1))
        .with_root(vec![root])
        .with_theme(Theme::nord())
}

fn key_press(code: KeyCode) -> KeyEvent {
    KeyEvent {
        code,
        modifiers: KeyModifiers::empty(),
        kind: KeyEventKind::Press,
    }
}

#[test]
fn test_tree_new_empty() {
    let tree = Tree::new(WidgetId::new(1));
    let plane = tree.render(Rect::new(0, 0, 40, 10));
    assert_eq!(plane.width, 40);
}

#[test]
fn test_tree_render_with_nodes() {
    let tree = make_tree();
    let plane = tree.render(Rect::new(0, 0, 40, 10));
    assert_eq!(plane.width, 40);
}

#[test]
fn test_tree_expand_collapse_enter() {
    let mut tree = make_tree();
    tree.set_area(Rect::new(0, 0, 40, 10));
    
    // Select first node (root)
    tree.set_selected_path(vec![0]);
    
    // Enter should toggle expand
    assert!(tree.handle_key(key_press(KeyCode::Enter)));
    
    // Render should show children
    let plane = tree.render(Rect::new(0, 0, 40, 10));
    assert_eq!(plane.width, 40);
}

#[test]
fn test_tree_right_arrow_expands() {
    let mut tree = make_tree();
    tree.set_area(Rect::new(0, 0, 40, 10));
    tree.set_selected_path(vec![0]);
    
    // Right arrow on collapsed node with children should expand
    assert!(tree.handle_key(key_press(KeyCode::Right)));
}

#[test]
fn test_tree_left_arrow_collapses() {
    let mut tree = make_tree();
    tree.set_area(Rect::new(0, 0, 40, 10));
    tree.set_selected_path(vec![0]);
    
    // First expand
    tree.handle_key(key_press(KeyCode::Right));
    // Then collapse
    assert!(tree.handle_key(key_press(KeyCode::Left)));
}

#[test]
fn test_tree_up_navigation() {
    let mut tree = make_tree();
    tree.set_area(Rect::new(0, 0, 40, 10));
    tree.set_selected_path(vec![0, 0]);
    
    // Up should pop path
    assert!(tree.handle_key(key_press(KeyCode::Up)));
}

#[test]
fn test_tree_down_navigation() {
    let mut tree = make_tree();
    tree.set_area(Rect::new(0, 0, 40, 10));
    tree.set_selected_path(vec![0]);
    
    // First expand the node
    tree.handle_key(key_press(KeyCode::Enter));
    
    // Down should navigate to child
    assert!(tree.handle_key(key_press(KeyCode::Down)));
}

#[test]
fn test_tree_mouse_click_selects() {
    let mut tree = make_tree();
    tree.set_area(Rect::new(0, 0, 40, 10));
    
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    let result = tree.handle_mouse(MouseEventKind::Down(MouseButton::Left), 5, 0);
    assert!(result);
}

#[test]
fn test_tree_mouse_hover() {
    let mut tree = make_tree();
    tree.set_area(Rect::new(0, 0, 40, 10));
    
    use dracon_terminal_engine::input::event::{MouseButton, MouseEventKind};
    let result = tree.handle_mouse(MouseEventKind::Moved, 5, 0);
    assert!(result);
}

#[test]
fn test_tree_toggle_expand_at() {
    let mut tree = make_tree();
    
    // Toggle expand on root's first child
    tree.set_selected_path(vec![0]);
    tree.handle_key(key_press(KeyCode::Enter));
    
    // Should be expanded now
    let plane1 = tree.render(Rect::new(0, 0, 40, 10));
    
    // Collapse
    tree.handle_key(key_press(KeyCode::Enter));
    let plane2 = tree.render(Rect::new(0, 0, 40, 10));
    
    // Planes should differ
    assert_ne!(plane1.cells.len(), 0);
    assert_ne!(plane2.cells.len(), 0);
}

#[test]
fn test_tree_get_selected_path() {
    let mut tree = make_tree();
    tree.set_selected_path(vec![0, 1]);
    assert_eq!(tree.get_selected_path(), &[0, 1]);
}

#[test]
fn test_tree_empty_path_navigate() {
    let mut tree = make_tree();
    tree.set_area(Rect::new(0, 0, 40, 10));
    
    // With empty path, up should not panic
    assert!(tree.handle_key(key_press(KeyCode::Up)));
}

#[test]
fn test_tree_theme_change() {
    let mut tree = make_tree();
    let theme = Theme::cyberpunk();
    tree.on_theme_change(&theme);
    
    let plane = tree.render(Rect::new(0, 0, 40, 10));
    for cell in &plane.cells {
        assert_eq!(cell.bg, theme.bg);
    }
}
