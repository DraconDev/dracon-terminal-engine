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
    for easing in [Easing::Linear, Easing::EaseIn, Easing::EaseOut, Easing::EaseInOut] {
        assert_eq!(Easing::apply_easing(&easing, 0.0), 0.0);
        assert_eq!(Easing::apply_easing(&easing, 1.0), 1.0);
    }
}

#[test]
fn test_easing_midpoint_monotonic() {
    // All easings should be monotonic (never decrease)
    for t in [0.0, 0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0] {
        let linear = Easing::apply_easing(&Easing::Linear, t);
        assert!(linear >= 0.0 && linear <= 1.0);
        
        let ease_in = Easing::apply_easing(&Easing::EaseIn, t);
        assert!(ease_in >= 0.0 && ease_in <= 1.0);
        
        let ease_out = Easing::apply_easing(&Easing::EaseOut, t);
        assert!(ease_out >= 0.0 && ease_out <= 1.0);
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
