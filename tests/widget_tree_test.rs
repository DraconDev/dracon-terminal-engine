//! Tests for the Tree widget.

use dracon_terminal_engine::framework::prelude::*;
use dracon_terminal_engine::framework::widget::Widget;
use dracon_terminal_engine::framework::widgets::tree::{Tree, TreeNode};

// ============================================================================
// Construction Tests
// ============================================================================

#[test]
fn test_tree_new() {
    let tree = Tree::new(WidgetId::new(1));
    let area = tree.area();
    assert!(area.width > 0);
}

#[test]
fn test_tree_with_root() {
    let mut root = TreeNode::new("Root");
    root.add_child(TreeNode::new("Child1"));
    root.add_child(TreeNode::new("Child2"));
    let tree = Tree::new(WidgetId::new(1)).with_root(vec![root]);
    let plane = tree.render(Rect::new(0, 0, 50, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_tree_with_theme() {
    let tree = Tree::new(WidgetId::new(1)).with_theme(Theme::nord());
    let plane = tree.render(Rect::new(0, 0, 50, 20));
    assert!(plane.width > 0);
}

#[test]
fn test_tree_empty() {
    let tree = Tree::new(WidgetId::new(1));
    let plane = tree.render(Rect::new(0, 0, 50, 20));
    assert!(plane.width > 0);
}

// ============================================================================
// Widget Trait Tests
// ============================================================================

#[test]
fn test_tree_id() {
    let tree = Tree::new(WidgetId::new(42));
    assert_eq!(tree.id(), WidgetId::new(42));
}

#[test]
fn test_tree_set_id() {
    let mut tree = Tree::new(WidgetId::new(1));
    tree.set_id(WidgetId::new(99));
    assert_eq!(tree.id(), WidgetId::new(99));
}

#[test]
fn test_tree_area() {
    let tree = Tree::new(WidgetId::new(1));
    let area = tree.area();
    assert!(area.width > 0);
}

#[test]
fn test_tree_set_area() {
    let mut tree = Tree::new(WidgetId::new(1));
    tree.set_area(Rect::new(0, 0, 80, 30));
    assert_eq!(tree.area(), Rect::new(0, 0, 80, 30));
}

#[test]
fn test_tree_needs_render() {
    let tree = Tree::new(WidgetId::new(1));
    assert!(tree.needs_render());
}

#[test]
fn test_tree_mark_dirty() {
    let mut tree = Tree::new(WidgetId::new(1));
    tree.clear_dirty();
    assert!(!tree.needs_render());
    tree.mark_dirty();
    assert!(tree.needs_render());
}

#[test]
fn test_tree_clear_dirty() {
    let mut tree = Tree::new(WidgetId::new(1));
    tree.clear_dirty();
    assert!(!tree.needs_render());
}

#[test]
fn test_tree_default_dirty() {
    let tree = Tree::new(WidgetId::new(1));
    assert!(tree.needs_render());
}

// ============================================================================
// Render Tests
// ============================================================================

#[test]
fn test_tree_render_basic() {
    let tree = Tree::new(WidgetId::new(1));
    let plane = tree.render(Rect::new(0, 0, 50, 20));
    assert_eq!(plane.width, 50);
}

#[test]
fn test_tree_render_has_content() {
    let mut root = TreeNode::new("Root");
    root.add_child(TreeNode::new("Child"));
    let tree = Tree::new(WidgetId::new(1)).with_root(vec![root]);
    let plane = tree.render(Rect::new(0, 0, 50, 20));
    let has_content = plane.cells.iter().any(|c| c.char != '\0');
    assert!(has_content);
}

#[test]
fn test_tree_render_wide() {
    let tree = Tree::new(WidgetId::new(1));
    let plane = tree.render(Rect::new(0, 0, 80, 20));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_tree_render_small() {
    let tree = Tree::new(WidgetId::new(1));
    let plane = tree.render(Rect::new(0, 0, 20, 10));
    assert_eq!(plane.width, 20);
}

#[test]
fn test_tree_render_tall() {
    let tree = Tree::new(WidgetId::new(1));
    let plane = tree.render(Rect::new(0, 0, 50, 40));
    assert_eq!(plane.height, 40);
}

// ============================================================================
// Theme Tests
// ============================================================================

#[test]
fn test_tree_on_theme_change() {
    let mut tree = Tree::new(WidgetId::new(1));
    tree.on_theme_change(&Theme::nord());
    assert!(tree.needs_render());
}

// ============================================================================
// Edge Cases
// ============================================================================

#[test]
fn test_tree_render_twice() {
    let tree = Tree::new(WidgetId::new(1));
    let _ = tree.render(Rect::new(0, 0, 50, 20));
    let _ = tree.render(Rect::new(0, 0, 50, 20));
}

#[test]
fn test_tree_set_area_then_render() {
    let mut tree = Tree::new(WidgetId::new(1));
    tree.set_area(Rect::new(0, 0, 80, 30));
    let plane = tree.render(Rect::new(0, 0, 80, 30));
    assert_eq!(plane.width, 80);
}

#[test]
fn test_tree_many_nodes() {
    let mut root = TreeNode::new("Root");
    for i in 0..10 {
        root.add_child(TreeNode::new(&format!("Child {}", i)));
    }
    let tree = Tree::new(WidgetId::new(1)).with_root(vec![root]);
    let plane = tree.render(Rect::new(0, 0, 50, 30));
    assert!(plane.width > 0);
}

#[test]
fn test_tree_nested_nodes() {
    let mut root = TreeNode::new("Level 0");
    let mut child = TreeNode::new("Level 1");
    child.add_child(TreeNode::new("Level 2"));
    root.add_child(child);
    let tree = Tree::new(WidgetId::new(1)).with_root(vec![root]);
    let plane = tree.render(Rect::new(0, 0, 50, 20));
    assert!(plane.width > 0);
}
