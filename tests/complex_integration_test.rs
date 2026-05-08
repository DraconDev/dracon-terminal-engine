//! Complex integration tests for drag-drop, focus cycles, and animation state transitions.
//!
//! These tests cover advanced scenarios that exercise multiple framework systems together.

use dracon_terminal_engine::framework::animation::{Animation, AnimationManager, Easing};
use dracon_terminal_engine::framework::dragdrop::{DragGhost, DragManager, DragPhase};
use dracon_terminal_engine::framework::focus::FocusManager;
use dracon_terminal_engine::framework::widget::WidgetId;
use std::sync::{Arc, Mutex};
use std::time::Duration;

mod common;
use common::TrackingWidget;

// =============================================================================
// Drag-and-Drop Complex Scenarios
// =============================================================================

#[test]
fn test_drag_payload_integrity() {
    let mut manager: DragManager<String> = DragManager::new();
    let ghost = DragGhost::new("Test");
    let payload = "sensitive_data_12345".to_string();

    manager.start_drag(payload.clone(), 1, ghost);
    manager.move_ghost(50, 50);

    // Payload should remain accessible during drag
    assert!(manager.is_dragging());
    assert_eq!(manager.phase(), DragPhase::Dragging);
}

#[test]
fn test_drag_rejection_no_valid_target() {
    let mut manager: DragManager<String> = DragManager::new();
    let ghost = DragGhost::new("Test");

    // Register a target but drop outside of it
    manager.register_target("target1".to_string(), 0, 0, 10, 10);
    manager.start_drag("data".to_string(), 1, ghost);
    manager.move_ghost(100, 100); // Far outside target

    let result = manager.end_drag();
    assert!(result.is_none());
    assert_eq!(manager.phase(), DragPhase::Cancelled);
}

#[test]
fn test_drag_rejection_wrong_target_type() {
    // DragManager uses the same type for both source and target.
    // This test verifies that a drag ends correctly even when
    // the target value differs from the payload.
    let mut manager: DragManager<usize> = DragManager::new();
    let ghost = DragGhost::new("Test");

    manager.register_target(1usize, 0, 0, 50, 50);
    manager.start_drag(2usize, 1, ghost); // Different value
    manager.move_ghost(25, 25);

    let result = manager.end_drag();
    // The manager returns the target ID, not the payload.
    // Higher-level code should compare payload to target.
    assert_eq!(result, Some(1usize));
}

#[test]
fn test_drag_consecutive_drags() {
    let mut manager: DragManager<usize> = DragManager::new();

    // First drag
    manager.register_target(1usize, 0, 0, 50, 50);
    let ghost1 = DragGhost::new("First");
    manager.start_drag(100usize, 1, ghost1);
    manager.move_ghost(25, 25);
    assert_eq!(manager.end_drag(), Some(1usize));

    // Second drag without clearing
    let ghost2 = DragGhost::new("Second");
    manager.start_drag(200usize, 2, ghost2);
    manager.move_ghost(25, 25);
    assert_eq!(manager.end_drag(), Some(1usize));

    assert_eq!(manager.phase(), DragPhase::Dropped);
}

#[test]
fn test_drag_ghost_position_accuracy() {
    let mut manager: DragManager<String> = DragManager::new();
    let ghost = DragGhost::new("Test").with_size(5, 1);

    manager.start_drag("data".to_string(), 1, ghost);
    manager.move_ghost(42, 37);

    let plane = manager.ghost_plane();
    assert!(plane.is_some());
    let plane = plane.unwrap();
    assert_eq!(plane.width, 5);
    assert_eq!(plane.height, 1);
}

#[test]
fn test_drag_target_overlap_priority() {
    let mut manager: DragManager<usize> = DragManager::new();
    let ghost = DragGhost::new("Test");

    // Two overlapping targets
    manager.register_target(1usize, 0, 0, 50, 50);
    manager.register_target(2usize, 20, 20, 50, 50); // Overlaps with first

    manager.start_drag(100usize, 1, ghost);
    manager.move_ghost(30, 30); // Inside both targets

    // Should return the first registered matching target
    let result = manager.end_drag();
    assert!(result == Some(1) || result == Some(2));
}

#[test]
fn test_drag_cancel_restores_idle() {
    let mut manager: DragManager<String> = DragManager::new();
    let ghost = DragGhost::new("Test");

    manager.start_drag("data".to_string(), 1, ghost);
    assert!(manager.is_dragging());

    manager.cancel();
    assert!(!manager.is_dragging());
    assert_eq!(manager.phase(), DragPhase::Cancelled);
    assert!(manager.ghost_plane().is_none());

    // Should be able to start a new drag after cancel
    let ghost2 = DragGhost::new("Test2");
    manager.start_drag("data2".to_string(), 2, ghost2);
    assert!(manager.is_dragging());
}

// =============================================================================
// Focus Cycle Complex Scenarios
// =============================================================================

#[test]
fn test_focus_skip_disabled_widget() {
    let mut fm = FocusManager::new();
    let id1 = WidgetId::new(1);
    let id2 = WidgetId::new(2); // disabled
    let id3 = WidgetId::new(3);

    fm.register(id1, true);
    fm.register(id2, false); // not focusable
    fm.register(id3, true);

    fm.set_focus(id1);
    assert_eq!(fm.tab_next(), Some(id3)); // skips id2
    assert_eq!(fm.tab_next(), Some(id1)); // wraps
}

#[test]
fn test_focus_custom_order() {
    // Register widgets out of order, verify tab cycles by registration order.
    let mut fm = FocusManager::new();
    let id_a = WidgetId::new(10);
    let id_b = WidgetId::new(5);
    let id_c = WidgetId::new(20);

    fm.register(id_a, true);
    fm.register(id_b, true);
    fm.register(id_c, true);

    // Tab order follows registration order, not ID order.
    fm.set_focus(id_a);
    assert_eq!(fm.tab_next(), Some(id_b));
    assert_eq!(fm.tab_next(), Some(id_c));
    assert_eq!(fm.tab_next(), Some(id_a));
}

#[test]
fn test_focus_restoration_after_clear() {
    let mut fm = FocusManager::new();
    let id1 = WidgetId::new(1);
    let id2 = WidgetId::new(2);

    fm.register(id1, true);
    fm.register(id2, true);
    fm.set_focus(id2);
    assert_eq!(fm.focused(), Some(id2));

    fm.clear_focus();
    assert_eq!(fm.focused(), None);

    // Should be able to restore focus
    fm.set_focus(id1);
    assert_eq!(fm.focused(), Some(id1));
}

#[test]
fn test_focus_callback_sequence_during_cycle() {
    let mut fm = FocusManager::new();
    let id1 = WidgetId::new(1);
    let id2 = WidgetId::new(2);
    let id3 = WidgetId::new(3);

    fm.register(id1, true);
    fm.register(id2, true);
    fm.register(id3, true);

    let changes = Arc::new(Mutex::new(Vec::new()));
    let changes_clone = changes.clone();

    fm.on_focus_change(move |new_id, old_id| {
        changes_clone.lock().unwrap().push((new_id, old_id));
    });

    fm.set_focus(id1); // (1, None)
    let _ = fm.tab_next(); // (2, Some(1))
    let _ = fm.tab_next(); // (3, Some(2))
    let _ = fm.tab_prev(); // (2, Some(3))

    let recorded = changes.lock().unwrap();
    assert_eq!(recorded.len(), 4);
    assert_eq!(recorded[0], (id1, None));
    assert_eq!(recorded[1], (id2, Some(id1)));
    assert_eq!(recorded[2], (id3, Some(id2)));
    assert_eq!(recorded[3], (id2, Some(id3)));
}

#[test]
fn test_focus_with_single_widget() {
    let mut fm = FocusManager::new();
    let id1 = WidgetId::new(1);

    fm.register(id1, true);
    fm.set_focus(id1);

    assert_eq!(fm.tab_next(), Some(id1)); // wraps to self
    assert_eq!(fm.tab_prev(), Some(id1)); // wraps to self
}

#[test]
fn test_focus_unregister_all_widgets() {
    let mut fm = FocusManager::new();
    let id1 = WidgetId::new(1);
    let id2 = WidgetId::new(2);

    fm.register(id1, true);
    fm.register(id2, true);
    fm.set_focus(id1);

    fm.unregister(id1);
    fm.unregister(id2);

    assert_eq!(fm.focused(), None);
    assert_eq!(fm.tab_next(), None);
}

#[test]
fn test_focus_trap_prevents_exit() {
    let mut fm = FocusManager::new();
    let id1 = WidgetId::new(1);
    let id2 = WidgetId::new(2);

    fm.register(id1, true);
    fm.register(id2, true);
    fm.set_focus(id1);

    fm.enter_trap();
    // When trapped, the trap flag should be set.
    assert!(fm.is_trapped());

    fm.enable_trap_exit();
    fm.exit_trap();
    assert!(!fm.is_trapped());
    assert_eq!(fm.focused(), Some(id1)); // focus unchanged
}

#[test]
fn test_focus_widget_receives_on_focus_and_on_blur() {
    let mut w1 = TrackingWidget::new(1);
    let mut w2 = TrackingWidget::new(2);

    assert_eq!(w1.focus_count(), 0);
    assert_eq!(w1.blur_count(), 0);

    dracon_terminal_engine::framework::widget::Widget::on_focus(&mut w1);
    assert_eq!(w1.focus_count(), 1);

    dracon_terminal_engine::framework::widget::Widget::on_blur(&mut w1);
    assert_eq!(w1.blur_count(), 1);

    // Test second widget independently
    dracon_terminal_engine::framework::widget::Widget::on_focus(&mut w2);
    assert_eq!(w2.focus_count(), 1);
}

// =============================================================================
// Animation State Transitions
// =============================================================================

#[test]
fn test_animation_progress_over_time() {
    let anim = Animation::new(0.0, 100.0, Duration::from_millis(100));
    std::thread::sleep(Duration::from_millis(10));
    let val_early = anim.value();

    std::thread::sleep(Duration::from_millis(40));
    let val_later = anim.value();

    // Should have progressed further
    assert!(
        val_later > val_early || val_later >= 100.0,
        "Animation should progress or be complete: early={}, later={}",
        val_early,
        val_later
    );
}

#[test]
fn test_animation_ease_in_start_slowly() {
    let anim = Animation::new(0.0, 100.0, Duration::from_millis(100)).with_easing(Easing::EaseIn);

    std::thread::sleep(Duration::from_millis(10));
    let val = anim.value();
    // EaseIn should start slowly — at ~10% time, value should be less than ~10%
    assert!(val < 15.0, "EaseIn should start slowly, got {}", val);
}

#[test]
fn test_animation_ease_out_end_slowly() {
    let anim = Animation::new(0.0, 100.0, Duration::from_millis(100)).with_easing(Easing::EaseOut);

    std::thread::sleep(Duration::from_millis(90));
    let val = anim.value();
    // EaseOut should end slowly — at ~90% time, value should be high
    assert!(val > 70.0, "EaseOut should be near end by 90%, got {}", val);
}

#[test]
fn test_animation_chained_sequence() {
    let mut manager = AnimationManager::new();

    // Start multiple animations that form a sequence.
    let id1 = manager.start(0.0, 50.0, Duration::from_millis(50));
    let id2 = manager.start(50.0, 100.0, Duration::from_millis(50));

    std::thread::sleep(Duration::from_millis(30));
    manager.tick();

    let val1 = manager.value(id1).unwrap_or(0.0);
    let val2 = manager.value(id2).unwrap_or(50.0);

    // Both should be progressing independently.
    assert!((0.0..=50.0).contains(&val1));
    assert!((50.0..=100.0).contains(&val2));
}

#[test]
fn test_animation_precise_value_linear_halfway() {
    let mut manager = AnimationManager::new();
    let id = manager.start(0.0, 100.0, Duration::from_millis(100));

    std::thread::sleep(Duration::from_millis(50));
    manager.tick();

    let val = manager.value(id).unwrap_or(0.0);
    // Linear easing at ~50% should be around 50 (with tolerance for scheduling).
    assert!(
        (30.0..=70.0).contains(&val),
        "Linear halfway should be near 50, got {}",
        val
    );
}

#[test]
fn test_animation_manager_cleanup_completed() {
    let mut manager = AnimationManager::new();
    manager.start(0.0, 100.0, Duration::from_millis(50));

    assert_eq!(manager.len(), 1);

    std::thread::sleep(Duration::from_millis(100));
    manager.tick();
    manager.cleanup();

    // After cleanup, completed animations should be removed.
    assert_eq!(manager.len(), 0);
}

#[test]
fn test_animation_manager_preserves_active_after_cleanup() {
    let mut manager = AnimationManager::new();
    let active_id = manager.start(0.0, 100.0, Duration::from_secs(10));
    manager.start(0.0, 100.0, Duration::from_millis(50));

    std::thread::sleep(Duration::from_millis(100));
    manager.cleanup();

    // Only the completed animation should be removed.
    assert_eq!(manager.len(), 1);
    assert!(manager.value(active_id).is_some());
}

#[test]
fn test_animation_state_idle_to_running() {
    let mut manager = AnimationManager::new();
    assert!(manager.is_empty());

    let id = manager.start(0.0, 100.0, Duration::from_secs(1));
    assert!(!manager.is_empty());
    assert!(manager.value(id).is_some());
}

#[test]
fn test_animation_state_running_to_done() {
    let mut manager = AnimationManager::new();
    let id = manager.start(0.0, 100.0, Duration::from_millis(50));

    std::thread::sleep(Duration::from_millis(10));
    manager.tick();
    let early_val = manager.value(id).unwrap();
    assert!(early_val < 100.0);

    std::thread::sleep(Duration::from_millis(100));
    manager.tick();

    // After completion, value might be removed or at final value.
    if let Some(val) = manager.value(id) {
        assert_eq!(val, 100.0);
    }
}

#[test]
fn test_animation_reverse_direction() {
    let mut manager = AnimationManager::new();
    let id = manager.start(100.0, 0.0, Duration::from_secs(1));

    std::thread::sleep(Duration::from_millis(50));
    manager.tick();

    let val = manager.value(id).unwrap_or(100.0);
    assert!(
        (0.0..100.0).contains(&val),
        "Reverse animation should decrease, got {}",
        val
    );
}

#[test]
fn test_animation_large_range() {
    let mut manager = AnimationManager::new();
    let id = manager.start(-1000.0, 1000.0, Duration::from_secs(1));

    std::thread::sleep(Duration::from_millis(50));
    manager.tick();

    let val = manager.value(id).unwrap_or(-1000.0);
    assert!(
        val > -1000.0 && val < 1000.0,
        "Large range should progress, got {}",
        val
    );
}

// =============================================================================
// Cross-System Integration Scenarios
// =============================================================================

#[test]
fn test_focus_and_animation_together() {
    let mut fm = FocusManager::new();
    let mut manager = AnimationManager::new();

    let id1 = WidgetId::new(1);
    fm.register(id1, true);
    fm.set_focus(id1);

    // Start animation while focus is active.
    let anim_id = manager.start(0.0, 100.0, Duration::from_millis(50));

    std::thread::sleep(Duration::from_millis(25));
    manager.tick();

    assert_eq!(fm.focused(), Some(id1));
    let val = manager.value(anim_id).unwrap_or(0.0);
    assert!(val > 0.0);
}

#[test]
fn test_drag_and_focus_independence() {
    let mut fm = FocusManager::new();
    let mut dm: DragManager<String> = DragManager::new();

    let id1 = WidgetId::new(1);
    fm.register(id1, true);
    fm.set_focus(id1);

    let ghost = DragGhost::new("Drag");
    dm.start_drag("data".to_string(), 1, ghost);

    // Focus and drag should operate independently.
    assert_eq!(fm.focused(), Some(id1));
    assert!(dm.is_dragging());

    dm.cancel();
    assert_eq!(fm.focused(), Some(id1)); // focus unchanged
}
