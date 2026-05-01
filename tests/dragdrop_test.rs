//! Tests for DragDrop system.

mod common;
use common::make_area;

use dracon_terminal_engine::framework::dragdrop::{DragGhost, DragManager, DragPhase, DropTarget};
use dracon_terminal_engine::framework::hitzone::DragState;

#[test]
fn test_drag_phase_variants() {
    assert_eq!(DragPhase::Idle, DragPhase::Idle);
    assert_eq!(DragPhase::Dragging, DragPhase::Dragging);
    assert_eq!(DragPhase::Dropped, DragPhase::Dropped);
    assert_eq!(DragPhase::Cancelled, DragPhase::Cancelled);
    assert_ne!(DragPhase::Idle, DragPhase::Dragging);
}

#[test]
fn test_drag_ghost_new() {
    let ghost = DragGhost::new("file.txt");
    assert_eq!(ghost.label, "file.txt");
    assert!(ghost.width >= 2);
    assert_eq!(ghost.height, 1);
}

#[test]
fn test_drag_ghost_with_size() {
    let ghost = DragGhost::new("file.txt").with_size(20, 3);
    assert_eq!(ghost.width, 20);
    assert_eq!(ghost.height, 3);
}

#[test]
fn test_drag_ghost_render() {
    let ghost = DragGhost::new("Hi").with_size(10, 1);
    let plane = ghost.render(5, 10);
    assert!(plane.width > 0);
    assert!(plane.height > 0);
}

#[test]
fn test_drag_manager_new() {
    let manager: DragManager<u32> = DragManager::new();
    assert_eq!(manager.phase(), DragPhase::Idle);
    assert!(!manager.is_dragging());
}

#[test]
fn test_drag_manager_clear() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.start_drag(1, 0, DragGhost::new("test"));
    manager.clear();
    assert_eq!(manager.phase(), DragPhase::Idle);
    assert!(!manager.is_dragging());
}

#[test]
fn test_drag_manager_start_drag() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.start_drag(42, 10, DragGhost::new("myfile"));
    assert_eq!(manager.phase(), DragPhase::Dragging);
    assert!(manager.is_dragging());
}

#[test]
fn test_drag_manager_move_ghost() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.move_ghost(100, 50);
}

#[test]
fn test_drag_manager_end_drag_no_target() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.start_drag(42, 10, DragGhost::new("test"));
    let result = manager.end_drag();
    assert_eq!(result, None);
    assert_eq!(manager.phase(), DragPhase::Cancelled);
}

#[test]
fn test_drag_manager_end_drag_on_target() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.register_target(99, 50, 50, 100, 20);
    manager.start_drag(42, 10, DragGhost::new("test"));
    manager.move_ghost(75, 55);
    let result = manager.end_drag();
    assert_eq!(result, Some(99));
    assert_eq!(manager.phase(), DragPhase::Dropped);
}

#[test]
fn test_drag_manager_end_drag_outside_target() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.register_target(99, 50, 50, 10, 10);
    manager.start_drag(42, 10, DragGhost::new("test"));
    manager.move_ghost(200, 200);
    let result = manager.end_drag();
    assert_eq!(result, None);
}

#[test]
fn test_drag_manager_cancel() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.start_drag(42, 10, DragGhost::new("test"));
    manager.cancel();
    assert_eq!(manager.phase(), DragPhase::Cancelled);
}

#[test]
fn test_drag_manager_register_target() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.register_target(1, 10, 20, 50, 30);
    manager.register_target(2, 100, 200, 50, 30);
    assert!(manager.ghost_plane().is_none());
}

#[test]
fn test_drag_manager_ghost_plane_none_when_idle() {
    let manager: DragManager<u32> = DragManager::new();
    assert!(manager.ghost_plane().is_none());
}

#[test]
fn test_drag_manager_ghost_plane_when_dragging() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.start_drag(42, 10, DragGhost::new("ghost"));
    let plane = manager.ghost_plane();
    assert!(plane.is_some());
}

#[test]
fn test_drag_manager_multiple_targets_first_match() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.register_target(1, 50, 50, 20, 20);
    manager.register_target(2, 70, 50, 20, 20);
    manager.start_drag(42, 10, DragGhost::new("test"));
    manager.move_ghost(55, 55);
    let result = manager.end_drag();
    assert_eq!(result, Some(1));
}

#[test]
fn test_drag_manager_target_at_boundary() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.register_target(99, 50, 50, 20, 20);
    manager.start_drag(42, 10, DragGhost::new("test"));
    manager.move_ghost(50, 50);
    let result = manager.end_drag();
    assert_eq!(result, Some(99));
}

#[test]
fn test_drag_manager_target_edge_exclusive() {
    let mut manager: DragManager<u32> = DragManager::new();
    manager.register_target(99, 50, 50, 20, 20);
    manager.start_drag(42, 10, DragGhost::new("test"));
    manager.move_ghost(70, 50);
    let result = manager.end_drag();
    assert_eq!(result, None);
}

#[test]
fn test_drop_target_clone() {
    let target = DropTarget::<u32> {
        id: 42,
        x: 10,
        y: 20,
        width: 50,
        height: 30,
    };
    let cloned = target.clone();
    assert_eq!(cloned.id, 42);
}

#[test]
fn test_drop_target_debug() {
    let target: DropTarget<i32> = DropTarget {
        id: 42,
        x: 10,
        y: 20,
        width: 50,
        height: 30,
    };
    let debug_str = format!("{:?}", target);
    assert!(debug_str.contains("42"));
}

#[test]
fn test_drag_state_debug() {
    let state = DragState::Started { x: 1, y: 2 };
    let debug_str = format!("{:?}", state);
    assert!(debug_str.contains("Started"));
}

#[test]
fn test_drag_ghost_debug() {
    let ghost = DragGhost::new("test");
    let debug_str = format!("{:?}", ghost);
    assert!(debug_str.contains("test"));
}
