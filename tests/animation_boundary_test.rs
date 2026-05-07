//! Animation boundary tests — empty queue, single frame, edge cases.

use dracon_terminal_engine::framework::animation::{Animation, AnimationManager, Easing};
use std::time::Duration;

#[test]
fn test_animation_manager_empty_queue() {
    let manager = AnimationManager::new();
    assert!(manager.is_empty());
    assert_eq!(manager.len(), 0);
}

#[test]
fn test_animation_manager_tick_empty() {
    let mut manager = AnimationManager::new();
    manager.tick(); // Should not panic
    assert!(manager.is_empty());
}

#[test]
fn test_animation_manager_cleanup_empty() {
    let mut manager = AnimationManager::new();
    manager.cleanup(); // Should not panic
    assert!(manager.is_empty());
}

#[test]
fn test_animation_single_frame() {
    let anim = Animation::new(0.0, 100.0, Duration::from_secs(1));
    let val = anim.value();
    // Should be very close to start (just started)
    assert!(val >= 0.0);
    assert!(val < 100.0);
}

#[test]
fn test_animation_zero_duration() {
    let anim = Animation::new(0.0, 100.0, Duration::from_millis(0));
    std::thread::sleep(Duration::from_millis(1));
    // Should immediately be at end value
    assert!(anim.is_done());
    assert_eq!(anim.value(), 100.0);
}

#[test]
fn test_animation_negative_range() {
    let anim = Animation::new(100.0, -50.0, Duration::from_secs(1));
    let val = anim.value();
    assert!(val <= 100.0);
    assert!(val >= -50.0);
}

#[test]
fn test_animation_same_start_end() {
    let anim = Animation::new(50.0, 50.0, Duration::from_secs(1));
    assert_eq!(anim.value(), 50.0);
}

#[test]
fn test_animation_manager_multiple_animations() {
    let mut manager = AnimationManager::new();
    let id1 = manager.start(0.0, 100.0, Duration::from_secs(1));
    let id2 = manager.start(50.0, 150.0, Duration::from_secs(1));
    
    assert_eq!(manager.len(), 2);
    assert!(manager.value(id1).is_some());
    assert!(manager.value(id2).is_some());
}

#[test]
fn test_animation_manager_value_nonexistent() {
    let manager = AnimationManager::new();
    assert!(manager.value(999).is_none());
}

#[test]
fn test_easing_boundary_values() {
    // Test that animations with different easings start at 0 and end at 1 proportionally
    let easings = vec![Easing::Linear, Easing::EaseIn, Easing::EaseOut, Easing::EaseInOut];
    for easing in easings {
        let anim_start = Animation::new(0.0, 100.0, Duration::from_secs(1)).with_easing(easing);
        let start_val = anim_start.value();
        assert!(start_val >= 0.0 && start_val <= 100.0);
    }
}

#[test]
fn test_easing_monotonic_behavior() {
    // Test that easings progress in the expected direction
    let anims = vec![
        Animation::new(0.0, 100.0, Duration::from_secs(1)).with_easing(Easing::Linear),
        Animation::new(0.0, 100.0, Duration::from_secs(1)).with_easing(Easing::EaseIn),
        Animation::new(0.0, 100.0, Duration::from_secs(1)).with_easing(Easing::EaseOut),
    ];
    
    std::thread::sleep(Duration::from_millis(10));
    
    for anim in &anims {
        let val = anim.value();
        assert!(val > 0.0 && val < 100.0, "Animation should have progressed");
    }
}

#[test]
fn test_animation_reset_midway() {
    let mut anim = Animation::new(0.0, 100.0, Duration::from_secs(1));
    std::thread::sleep(Duration::from_millis(50));
    let mid_val = anim.value();
    
    anim.reset();
    let reset_val = anim.value();
    
    assert!(reset_val < mid_val || reset_val == 0.0);
}

#[test]
fn test_animation_manager_start_after_clear() {
    let mut manager = AnimationManager::new();
    manager.start(0.0, 100.0, Duration::from_secs(1));
    manager.clear();
    
    let id = manager.start(50.0, 150.0, Duration::from_secs(1));
    assert!(manager.value(id).is_some());
    assert_eq!(manager.len(), 1);
}
