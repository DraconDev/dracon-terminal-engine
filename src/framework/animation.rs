//! Animation utility for tweening values over time.
//!
//! Provides `Animation` which interpolates between start and end values
//! over a configurable duration with easing functions.

use std::time::{Duration, Instant};

/// Easing function for animation curves.
pub enum Easing {
    /// Linear easing.
    Linear,
    /// Ease-in easing.
    EaseIn,
    /// Ease-out easing.
    EaseOut,
    /// Ease-in-out easing.
    EaseInOut,
}

impl Easing {
    fn apply_easing(easing: &Easing, t: f64) -> f64 {
        match easing {
            Easing::Linear => t,
            Easing::EaseIn => t * t,
            Easing::EaseOut => t * (2.0 - t),
            Easing::EaseInOut => {
                if t < 0.5 {
                    2.0 * t * t
                } else {
                    -1.0 + (4.0 - 2.0 * t) * t
                }
            }
        }
    }
}

/// An active animation that tweens a value from start to end.
pub struct Animation {
    start_value: f64,
    end_value: f64,
    start_time: Instant,
    duration: Duration,
    easing: Easing,
    completed: bool,
}

impl Animation {
    /// Creates a new animation from start to end over the given duration.
    pub fn new(start: f64, end: f64, duration: Duration) -> Self {
        Self {
            start_value: start,
            end_value: end,
            start_time: Instant::now(),
            duration,
            easing: Easing::Linear,
            completed: false,
        }
    }

    /// Sets the easing function for this animation.
    pub fn with_easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Returns the current interpolated value.
    pub fn value(&self) -> f64 {
        if self.completed {
            return self.end_value;
        }
        let elapsed = Instant::now().duration_since(self.start_time);
        let t = (elapsed.as_secs_f64() / self.duration.as_secs_f64()).min(1.0);
        let eased = Easing::apply_easing(&self.easing, t);
        self.start_value + (self.end_value - self.start_value) * eased
    }

    /// Returns true if the animation has completed.
    pub fn is_done(&self) -> bool {
        self.completed || Instant::now().duration_since(self.start_time) >= self.duration
    }

    /// Resets the animation to the start value.
    pub fn reset(&mut self) {
        self.start_time = Instant::now();
        self.completed = false;
    }
}

/// Manages multiple active animations.
pub struct AnimationManager {
    animations: Vec<Animation>,
}

impl AnimationManager {
    /// Creates a new animation manager.
    pub fn new() -> Self {
        Self {
            animations: Vec::new(),
        }
    }

    /// Starts a new animation and returns its index.
    pub fn start(&mut self, start: f64, end: f64, duration: Duration) -> usize {
        let id = self.animations.len();
        self.animations.push(Animation::new(start, end, duration));
        id
    }

    /// Gets the current value of an animation by index.
    pub fn value(&self, id: usize) -> Option<f64> {
        self.animations.get(id).map(|a| a.value())
    }

    /// Returns true if an animation is done.
    pub fn is_done(&self, id: usize) -> bool {
        self.animations.get(id).map(|a| a.is_done()).unwrap_or(true)
    }

    /// Removes completed animations.
    pub fn cleanup(&mut self) {
        self.animations.retain(|a| !a.is_done());
    }

    /// Clears all animations.
    pub fn clear(&mut self) {
        self.animations.clear();
    }

    /// Advances all animations by cleaning up completed ones.
    /// Call this each frame.
    pub fn tick(&mut self) {
        self.cleanup();
    }

    /// Returns the number of active animations.
    pub fn len(&self) -> usize {
        self.animations.len()
    }

    /// Returns true if there are no active animations.
    pub fn is_empty(&self) -> bool {
        self.animations.is_empty()
    }
}

impl Default for AnimationManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_animation_linear() {
        let anim = Animation::new(0.0, 100.0, Duration::from_secs(1));
        let _ = anim.value();
        assert!(!anim.is_done());
    }

    #[test]
    fn test_animation_easing() {
        let anim = Animation::new(0.0, 1.0, Duration::from_secs(1)).with_easing(Easing::EaseIn);
        let _ = anim.value();
        assert!(!anim.is_done());
    }

    #[test]
    fn test_animation_manager() {
        let mut manager = AnimationManager::new();
        let id = manager.start(0.0, 100.0, Duration::from_secs(1));
        assert!(manager.value(id).is_some());
        assert!(!manager.is_done(id));
    }

    #[test]
    fn test_animation_value_at_start() {
        let anim = Animation::new(10.0, 20.0, Duration::from_secs(1));
        std::thread::sleep(Duration::from_millis(5));
        let val = anim.value();
        assert!((10.0..=20.0).contains(&val));
    }

    #[test]
    fn test_animation_completes_after_duration() {
        let anim = Animation::new(0.0, 100.0, Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(60));
        assert!(anim.is_done());
        assert_eq!(anim.value(), 100.0);
    }

    #[test]
    fn test_animation_reset() {
        let mut anim = Animation::new(0.0, 100.0, Duration::from_secs(1));
        std::thread::sleep(Duration::from_millis(50));
        anim.reset();
        assert!(!anim.is_done());
        assert!(anim.value() < 10.0);
    }

    #[test]
    fn test_animation_easing_values() {
        let ease_in = Easing::EaseIn;
        let ease_out = Easing::EaseOut;
        let _ease_in_out = Easing::EaseInOut;
        let linear = Easing::Linear;

        assert_eq!(Easing::apply_easing(&linear, 0.0), 0.0);
        assert_eq!(Easing::apply_easing(&linear, 1.0), 1.0);
        assert!(Easing::apply_easing(&ease_in, 0.5) < 0.5);
        assert!(Easing::apply_easing(&ease_out, 0.5) > 0.5);
    }

    #[test]
    fn test_animation_manager_cleanup() {
        let mut manager = AnimationManager::new();
        let id = manager.start(0.0, 100.0, Duration::from_millis(1));
        assert!(manager.value(id).is_some());
        std::thread::sleep(Duration::from_millis(10));
        manager.cleanup();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_animation_manager_clear() {
        let mut manager = AnimationManager::new();
        manager.start(0.0, 100.0, Duration::from_secs(1));
        manager.start(0.0, 50.0, Duration::from_secs(1));
        assert_eq!(manager.len(), 2);
        manager.clear();
        assert!(manager.is_empty());
    }

    #[test]
    fn test_animation_manager_is_done_nonexistent() {
        let manager = AnimationManager::new();
        assert!(manager.is_done(999));
        assert!(manager.value(999).is_none());
    }

    // ===== Comprehensive Interpolation Tests =====

    #[test]
    fn test_easing_linear_at_quarter_half_three_quarter() {
        let linear = Easing::Linear;
        assert_eq!(Easing::apply_easing(&linear, 0.0), 0.0);
        assert_eq!(Easing::apply_easing(&linear, 0.25), 0.25);
        assert_eq!(Easing::apply_easing(&linear, 0.5), 0.5);
        assert_eq!(Easing::apply_easing(&linear, 0.75), 0.75);
        assert_eq!(Easing::apply_easing(&linear, 1.0), 1.0);
    }

    #[test]
    fn test_easing_ease_in_quadratic_curve() {
        let ease_in = Easing::EaseIn;
        assert_eq!(Easing::apply_easing(&ease_in, 0.0), 0.0);
        assert!((Easing::apply_easing(&ease_in, 0.5) - 0.25).abs() < 0.0001);
        assert!((Easing::apply_easing(&ease_in, 0.0) - 0.0).abs() < 0.0001);
        assert!((Easing::apply_easing(&ease_in, 1.0) - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_easing_ease_out_decay_curve() {
        let ease_out = Easing::EaseOut;
        assert_eq!(Easing::apply_easing(&ease_out, 0.0), 0.0);
        assert_eq!(Easing::apply_easing(&ease_out, 1.0), 1.0);
        let val_at_half = Easing::apply_easing(&ease_out, 0.5);
        assert!(val_at_half > 0.5);
        assert!(val_at_half < 1.0);
        let val_at_quarter = Easing::apply_easing(&ease_out, 0.25);
        assert!(val_at_quarter > 0.25);
    }

    #[test]
    fn test_easing_ease_in_out_symmetric() {
        let ease_in_out = Easing::EaseInOut;
        assert_eq!(Easing::apply_easing(&ease_in_out, 0.0), 0.0);
        assert_eq!(Easing::apply_easing(&ease_in_out, 1.0), 1.0);
        let at_quarter = Easing::apply_easing(&ease_in_out, 0.25);
        let at_half = Easing::apply_easing(&ease_in_out, 0.5);
        let at_three_quarter = Easing::apply_easing(&ease_in_out, 0.75);
        assert!(at_quarter < at_half);
        assert!(at_half < at_three_quarter);
        assert!((at_quarter + at_three_quarter - 1.0).abs() < 0.0001);
    }

    #[test]
    fn test_interpolation_with_negative_values() {
        let anim = Animation::new(-100.0, 100.0, Duration::from_secs(1));
        std::thread::sleep(Duration::from_millis(10));
        let val = anim.value();
        assert!((-100.0..=100.0).contains(&val));
    }

    #[test]
    fn test_interpolation_with_start_greater_than_end() {
        let anim = Animation::new(100.0, 0.0, Duration::from_secs(1));
        std::thread::sleep(Duration::from_millis(10));
        let val = anim.value();
        assert!((0.0..=100.0).contains(&val));
        assert!(anim.value() < 100.0);
    }

    #[test]
    fn test_interpolation_zero_duration_immediately_complete() {
        let anim = Animation::new(0.0, 100.0, Duration::from_millis(0));
        std::thread::sleep(Duration::from_millis(1));
        assert!(anim.is_done());
        assert_eq!(anim.value(), 100.0);
    }

    #[test]
    fn test_animation_value_clamped_to_end_after_completion() {
        let anim = Animation::new(0.0, 100.0, Duration::from_millis(50));
        std::thread::sleep(Duration::from_millis(100));
        let values: Vec<f64> = (0..5).map(|_| anim.value()).collect();
        for val in values {
            assert_eq!(val, 100.0);
        }
    }

    #[test]
    fn test_manager_multiple_simultaneous_animations() {
        let mut manager = AnimationManager::new();
        let id1 = manager.start(0.0, 100.0, Duration::from_secs(2));
        let id2 = manager.start(0.0, 50.0, Duration::from_secs(1));
        let id3 = manager.start(100.0, 200.0, Duration::from_secs(3));
        assert_eq!(manager.len(), 3);
        let val1 = manager.value(id1);
        let val2 = manager.value(id2);
        let val3 = manager.value(id3);
        assert!(val1.is_some());
        assert!(val2.is_some());
        assert!(val3.is_some());
        assert!(val1.unwrap() < 100.0);
        assert!(val2.unwrap() < 50.0);
        assert!(val3.unwrap() > 100.0 && val3.unwrap() <= 200.0);
    }

    #[test]
    fn test_manager_preserves_active_after_cleanup() {
        let mut manager = AnimationManager::new();
        manager.start(0.0, 100.0, Duration::from_millis(10));
        manager.start(0.0, 100.0, Duration::from_secs(10));
        std::thread::sleep(Duration::from_millis(50));
        manager.cleanup();
        assert_eq!(manager.len(), 1);
        assert!(!manager.is_empty());
    }

    #[test]
    fn test_animation_with_easing_applies_correctly() {
        let anim = Animation::new(0.0, 100.0, Duration::from_secs(1)).with_easing(Easing::EaseIn);
        let val1 = anim.value();
        std::thread::sleep(Duration::from_millis(250));
        let val2 = anim.value();
        assert!(val2 > val1);
        assert!(val2 < 100.0);
    }

    #[test]
    fn test_easing_boundary_conditions() {
        let ease_in = Easing::EaseIn;
        let ease_out = Easing::EaseOut;
        let ease_in_out = Easing::EaseInOut;
        let linear = Easing::Linear;
        for easing in [&linear, &ease_in, &ease_out, &ease_in_out] {
            assert!((Easing::apply_easing(easing, 0.0) - 0.0).abs() < 0.0001);
            assert!((Easing::apply_easing(easing, 1.0) - 1.0).abs() < 0.0001);
        }
    }

    #[test]
    fn test_animation_is_done_flag_prevents_negative_progress() {
        let anim = Animation::new(0.0, 100.0, Duration::from_secs(1));
        std::thread::sleep(Duration::from_millis(1100));
        assert!(anim.is_done());
        let val = anim.value();
        assert_eq!(val, 100.0);
    }
}
