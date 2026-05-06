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
        assert!(val >= 10.0 && val <= 20.0);
    }

    #[test]
    fn test_animation_completes_after_duration() {
        let mut anim = Animation::new(0.0, 100.0, Duration::from_millis(50));
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
        let ease_in_out = Easing::EaseInOut;
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
}
