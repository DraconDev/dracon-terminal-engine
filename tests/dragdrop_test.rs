//! Drag and drop system tests.

use dracon_terminal_engine::framework::dragdrop::{DragGhost, DragManager, DragPhase};

#[test]
fn test_drag_manager_new_is_idle() {
    let manager: DragManager<String> = DragManager::new();
    assert_eq!(manager.phase(), DragPhase::Idle);
    assert!(!manager.is_dragging());
}

#[test]
fn test_drag_manager_start_drag() {
    let mut manager: DragManager<String> = DragManager::new();
    let ghost = DragGhost::new("Test Item");
    
    manager.start_drag("data".to_string(), 1, ghost);
    assert_eq!(manager.phase(), DragPhase::Dragging);
    assert!(manager.is_dragging());
}

#[test]
fn test_drag_manager_move_ghost() {
    let mut manager: DragManager<String> = DragManager::new();
    let ghost = DragGhost::new("Test");
    
    manager.start_drag("data".to_string(), 1, ghost);
    manager.move_ghost(50, 30);
    
    let plane = manager.ghost_plane();
    assert!(plane.is_some());
}

#[test]
fn test_drag_manager_end_drag_no_target() {
    let mut manager: DragManager<String> = DragManager::new();
    let ghost = DragGhost::new("Test");
    
    manager.start_drag("data".to_string(), 1, ghost);
    manager.move_ghost(100, 100);
    
    let result = manager.end_drag();
    assert!(result.is_none());
    assert_eq!(manager.phase(), DragPhase::Cancelled);
    assert!(!manager.is_dragging());
}

#[test]
fn test_drag_manager_end_drag_with_target() {
    let mut manager: DragManager<String> = DragManager::new();
    let ghost = DragGhost::new("Test");
    
    manager.register_target("target1".to_string(), 10, 10, 50, 50);
    manager.start_drag("data".to_string(), 1, ghost);
    manager.move_ghost(30, 30); // Inside target
    
    let result = manager.end_drag();
    assert_eq!(result, Some("target1".to_string()));
    assert_eq!(manager.phase(), DragPhase::Dropped);
}

#[test]
fn test_drag_manager_cancel() {
    let mut manager: DragManager<String> = DragManager::new();
    let ghost = DragGhost::new("Test");
    
    manager.start_drag("data".to_string(), 1, ghost);
    manager.cancel();
    
    assert_eq!(manager.phase(), DragPhase::Cancelled);
    assert!(!manager.is_dragging());
    assert!(manager.ghost_plane().is_none());
}

#[test]
fn test_drag_manager_clear() {
    let mut manager: DragManager<String> = DragManager::new();
    let ghost = DragGhost::new("Test");
    
    manager.register_target("t1".to_string(), 0, 0, 10, 10);
    manager.start_drag("data".to_string(), 1, ghost);
    manager.clear();
    
    assert_eq!(manager.phase(), DragPhase::Idle);
}

#[test]
fn test_drag_ghost_new() {
    let ghost = DragGhost::new("Item");
    assert_eq!(ghost.label, "Item");
    assert_eq!(ghost.height, 1);
    assert!(ghost.width > 0);
}

#[test]
fn test_drag_ghost_with_size() {
    let ghost = DragGhost::new("Item").with_size(20, 3);
    assert_eq!(ghost.width, 20);
    assert_eq!(ghost.height, 3);
}

#[test]
fn test_drag_ghost_render() {
    let ghost = DragGhost::new("Test");
    let plane = ghost.render(10, 20);
    
    assert_eq!(plane.width, ghost.width);
    assert_eq!(plane.height, ghost.height);
}

#[test]
fn test_drag_manager_multiple_targets() {
    let mut manager: DragManager<usize> = DragManager::new();
    let ghost = DragGhost::new("Test");
    
    manager.register_target(1, 0, 0, 50, 50);
    manager.register_target(2, 100, 100, 50, 50);
    
    manager.start_drag("data".to_string(), 1, ghost);
    manager.move_ghost(25, 25); // Inside target 1
    
    assert_eq!(manager.end_drag(), Some(1));
}

#[test]
fn test_drag_manager_target_edge_boundary() {
    let mut manager: DragManager<usize> = DragManager::new();
    let ghost = DragGhost::new("Test");
    
    manager.register_target(1, 10, 10, 20, 20);
    
    manager.start_drag("data".to_string(), 1, ghost);
    manager.move_ghost(10, 10); // Top-left edge
    
    assert_eq!(manager.end_drag(), Some(1));
}
