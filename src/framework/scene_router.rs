//! Scene Router — Multi-screen navigation for Dracon apps.
//!
//! Provides stack-based screen management so apps can have multiple
//! "pages" with push/pop navigation, transitions, and lifecycle hooks.
//!
//! ## Example
//!
//! ```no_run
//! use dracon_terminal_engine::framework::scene_router::{Scene, SceneRouter};
//! use dracon_terminal_engine::compositor::Plane;
//! use dracon_terminal_engine::framework::theme::Theme;
//! use dracon_terminal_engine::input::event::{KeyEvent, MouseEventKind};
//! use ratatui::layout::Rect;
//!
//! struct HomeScreen;
//! impl Scene for HomeScreen {
//!     fn scene_id(&self) -> &str { "home" }
//!     fn render(&self, area: Rect) -> Plane { Plane::new(0, area.width, area.height) }
//!     fn handle_key(&mut self, _key: KeyEvent) -> bool { false }
//!     fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool { false }
//!     fn needs_render(&self) -> bool { true }
//!     fn mark_dirty(&mut self) {}
//!     fn clear_dirty(&mut self) {}
//! }
//!
//! let mut router = SceneRouter::new();
//! router.register("home", Box::new(HomeScreen));
//! router.push("home");
//! ```

use crate::compositor::Plane;
use crate::framework::theme::Theme;
use crate::input::event::{KeyEvent, MouseEventKind};
use ratatui::layout::Rect;
use std::any::Any;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::time::Instant;

// ═══════════════════════════════════════════════════════════════════════════════
// SCENE TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// A screen in the application.
///
/// Scenes are like Widgets but with navigation lifecycle hooks.
/// Implement this on your app screens.
pub trait Scene: Any {
    /// Unique identifier for this scene type.
    fn scene_id(&self) -> &str;

    /// Called when this scene becomes the active screen.
    fn on_enter(&mut self) {}

    /// Called when this scene is no longer active (pushed to stack).
    fn on_exit(&mut self) {}

    /// Called when returning to this scene from the stack.
    fn on_resume(&mut self) {}

    /// Called when this scene is covered by a new scene.
    fn on_pause(&mut self) {}

    /// Render the scene. Similar to Widget::render.
    fn render(&self, area: Rect) -> Plane;

    /// Handle keyboard input. Return true if handled.
    fn handle_key(&mut self, key: KeyEvent) -> bool;

    /// Handle mouse input. Return true if handled.
    fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool;

    /// Called when the app theme changes.
    fn on_theme_change(&mut self, _theme: &Theme) {}

    /// Whether this scene needs a re-render.
    fn needs_render(&self) -> bool;

    /// Mark this scene as dirty (needs re-render).
    fn mark_dirty(&mut self);

    /// Clear the dirty flag.
    fn clear_dirty(&mut self);
}

/// Available scene transition animations.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SceneTransition {
    /// Fade out old scene, fade in new scene.
    Fade,
    /// Slide new scene in from the right.
    SlideLeft,
    /// Slide new scene in from the left.
    SlideRight,
    /// Slide new scene in from the bottom.
    SlideUp,
    /// Slide new scene in from the top.
    SlideDown,
    /// No transition (instant switch).
    None,
}

/// Active transition state.
struct TransitionState {
    from_scene: String,
    to_scene: String,
    progress: f32,
    duration_ms: f32,
    elapsed_ms: f32,
    transition: SceneTransition,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NAVIGATION EVENTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Events published by the SceneRouter on navigation changes.
#[derive(Clone, Debug)]
pub enum NavigationEvent {
    /// Navigated to a new scene. (from_scene, to_scene)
    Navigated(Option<String>, String),
    /// Returned to a previous scene. (from_scene, to_scene)
    Popped(String, String),
    /// Replaced the current scene. (old_scene, new_scene)
    Replaced(String, String),
}

// ═══════════════════════════════════════════════════════════════════════════════
// SCENE ROUTER
// ═══════════════════════════════════════════════════════════════════════════════

/// Manages a stack of scenes and handles navigation between them.
///
/// The router maintains a registry of all available scenes and a navigation
/// stack. Only the top scene on the stack is active.
///
/// # Interior Mutability
///
/// SceneRouter uses interior mutability for `transition` and `dirty` fields
/// so that `render(&self)` can advance transitions without requiring `&mut self`.
/// This allows the router to be used from Widget::render which takes `&self`.
pub struct SceneRouter {
    /// Registry of all available scenes by ID.
    scenes: HashMap<String, Box<dyn Scene>>,
    /// Navigation stack. Top is current scene.
    stack: Vec<String>,
    /// Whether the current scene needs rendering.
    dirty: Cell<bool>,
    /// Theme reference for propagation.
    theme: Option<Theme>,
    /// Active transition (if any).
    transition: RefCell<Option<TransitionState>>,
    /// Default transition for all navigation.
    default_transition: SceneTransition,
    /// Default transition duration in milliseconds.
    default_duration_ms: f32,
    /// Last render timestamp for delta time calculation.
    last_render: RefCell<Option<Instant>>,
}

impl SceneRouter {
    /// Creates a new SceneRouter.
    pub fn new() -> Self {
        Self {
            scenes: HashMap::new(),
            stack: Vec::new(),
            dirty: Cell::new(false),
            theme: None,
            transition: RefCell::new(None),
            default_transition: SceneTransition::Fade,
            default_duration_ms: 200.0,
            last_render: RefCell::new(None),
        }
    }

    // ── Registration ────────────────────────────────────────────────────────

    /// Registers a scene with the router.
    ///
    /// The scene's `scene_id()` is used as the key.
    pub fn register(&mut self, id: &str, scene: Box<dyn Scene>) {
        self.scenes.insert(id.to_string(), scene);
    }

    /// Unregisters a scene by ID.
    pub fn unregister(&mut self, id: &str) -> Option<Box<dyn Scene>> {
        self.scenes.remove(id)
    }

    /// Checks if a scene is registered.
    pub fn has_scene(&self, id: &str) -> bool {
        self.scenes.contains_key(id)
    }

    // ── Navigation ──────────────────────────────────────────────────────────

    /// Sets the default transition for all navigation.
    pub fn with_default_transition(mut self, transition: SceneTransition) -> Self {
        self.default_transition = transition;
        self
    }

    /// Sets the default transition duration in milliseconds.
    pub fn with_default_duration(mut self, ms: f32) -> Self {
        self.default_duration_ms = ms;
        self
    }

    /// Pushes a scene with a specific transition.
    pub fn push_with_transition(&mut self, id: &str, transition: SceneTransition, duration_ms: f32) {
        if !self.scenes.contains_key(id) {
            return;
        }

        // Start transition if we have a current scene
        if let Some(current_id) = self.stack.last().cloned() {
            if transition != SceneTransition::None {
                *self.transition.borrow_mut() = Some(TransitionState {
                    from_scene: current_id,
                    to_scene: id.to_string(),
                    progress: 0.0,
                    duration_ms,
                    elapsed_ms: 0.0,
                    transition,
                });
            }
        }

        // Pause current scene
        if let Some(current_id) = self.stack.last() {
            if let Some(scene) = self.scenes.get_mut(current_id) {
                scene.on_pause();
            }
        }

        self.stack.push(id.to_string());

        // Enter new scene
        if let Some(scene) = self.scenes.get_mut(id) {
            scene.on_enter();
            if let Some(ref theme) = self.theme {
                scene.on_theme_change(theme);
            }
        }

        self.dirty.set(true);
    }

    /// Pushes a scene onto the navigation stack with the default transition.
    pub fn push(&mut self, id: &str) {
        let transition = self.default_transition;
        let duration = self.default_duration_ms;
        self.push_with_transition(id, transition, duration);
    }

    /// Pops the current scene and returns to the previous one.
    ///
    /// Returns true if a scene was popped. Returns false if the stack
    /// only has one scene (can't pop the root).
    pub fn pop(&mut self) -> bool {
        if self.stack.len() <= 1 {
            return false;
        }

        let from = self.stack.pop().expect("scene_router: stack non-empty on pop (len > 1)");
        if let Some(scene) = self.scenes.get_mut(&from) {
            scene.on_exit();
        }

        // Resume previous scene
        if let Some(to) = self.stack.last() {
            let to = to.clone();
            if let Some(scene) = self.scenes.get_mut(&to) {
                scene.on_resume();
            }
        }

        self.dirty.set(true);
        true
    }

    /// Pops the current scene even if it's the only one on the stack.
    ///
    /// Unlike [`pop()`](Self::pop), this does not guard against popping the root scene.
    /// Use this when the caller wants to return to a non-scene state
    /// (e.g., the showcase launcher) by clearing the stack entirely.
    ///
    /// Returns `true` if a scene was popped, `false` if the stack was empty.
    ///
    /// # When to use `pop()` vs `pop_force()`
    ///
    /// | Method | Behavior when stack has 1 scene |
    /// |--------|-------------------------------|
    /// | `pop()` | Returns `false` (guards root) |
    /// | `pop_force()` | Pops the root scene |
    pub fn pop_force(&mut self) -> bool {
        if self.stack.is_empty() {
            return false;
        }

        let from = self.stack.pop().expect("scene_router: stack non-empty on pop");

        if let Some(scene) = self.scenes.get_mut(&from) {
            scene.on_exit();
        }

        if let Some(to) = self.stack.last() {
            let to = to.clone();
            if let Some(scene) = self.scenes.get_mut(&to) {
                scene.on_resume();
            }
        }

        self.dirty.set(true);
        true
    }

    /// Replaces the current scene with a new one.
    ///
    /// The old scene receives `on_exit`. The new scene receives `on_enter`.
    pub fn replace(&mut self, id: &str) {
        if !self.scenes.contains_key(id) {
            return;
        }

        if let Some(old_id) = self.stack.last().cloned() {
            if let Some(scene) = self.scenes.get_mut(&old_id) {
                scene.on_exit();
            }
        }

        if let Some(last) = self.stack.last_mut() {
            *last = id.to_string();
        } else {
            self.stack.push(id.to_string());
        }

        if let Some(scene) = self.scenes.get_mut(id) {
            scene.on_enter();
            if let Some(ref theme) = self.theme {
                scene.on_theme_change(theme);
            }
        }

        self.dirty.set(true);
    }

    /// Navigates to a scene without pushing to the stack.
    ///
    /// Clears the stack and sets this as the only scene.
    pub fn go(&mut self, id: &str) {
        if !self.scenes.contains_key(id) {
            return;
        }

        // Exit all scenes
        for scene_id in &self.stack {
            if let Some(scene) = self.scenes.get_mut(scene_id) {
                scene.on_exit();
            }
        }

        self.stack.clear();
        self.stack.push(id.to_string());

        if let Some(scene) = self.scenes.get_mut(id) {
            scene.on_enter();
            if let Some(ref theme) = self.theme {
                scene.on_theme_change(theme);
            }
        }

        self.dirty.set(true);
    }

    /// Navigates to a deep-linked path like "settings/profile".
    ///
    /// Splits the path by '/' and pushes each scene segment.
    /// If a segment is not registered, navigation stops there.
    pub fn navigate_to(&mut self, path: &str) {
        let segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
        if segments.is_empty() {
            return;
        }

        // Go to the first segment (clears stack)
        self.go(segments[0]);

        // Push remaining segments
        for segment in &segments[1..] {
            self.push(segment);
        }
    }

    /// Returns the current navigation path as a slash-separated string.
    pub fn current_path(&self) -> String {
        self.stack.join("/")
    }

    // ── State Queries ───────────────────────────────────────────────────────

    /// Returns the ID of the current scene, if any.
    pub fn current(&self) -> Option<&str> {
        self.stack.last().map(|s| s.as_str())
    }

    /// Returns the current stack depth.
    pub fn stack_depth(&self) -> usize {
        self.stack.len()
    }

    /// Returns true if the stack can be popped (more than 1 scene).
    pub fn can_go_back(&self) -> bool {
        self.stack.len() > 1
    }

    /// Returns true if a scene with the given ID is registered.
    pub fn is_registered(&self, id: &str) -> bool {
        self.scenes.contains_key(id)
    }

    /// Returns a reference to a registered scene by ID.
    pub fn get_scene(&self, id: &str) -> Option<&dyn Scene> {
        self.scenes.get(id).map(|s| s.as_ref())
    }

    /// Returns a mutable reference to a registered scene by ID.
    pub fn get_scene_mut(&mut self, id: &str) -> Option<&mut dyn Scene> {
        self.scenes.get_mut(id).map(|s| s.as_mut())
    }

    // ── Rendering ───────────────────────────────────────────────────────────

    /// Advances any active transition and delegates render to the current scene.
    ///
    /// Transition timing is automatically calculated using actual wall-clock delta
    /// time between frames. For apps with different frame rates, transitions will
    /// take the same real-world duration regardless of frame rate.
    pub fn render(&self, area: Rect) -> Plane {
        // Calculate actual delta time since last render
        let dt_ms = {
            let now = Instant::now();
            let mut last = self.last_render.borrow_mut();
            let dt = last.map(|t| {
                let elapsed = now.duration_since(t);
                elapsed.as_secs_f32() * 1000.0
            }).unwrap_or(16.0); // Default 16ms on first frame
            *last = Some(now);
            dt
        };

        // Advance transition if active
        let transition_active = self.tick_transition_internal(dt_ms);

        if transition_active {
            let trans = self.transition.borrow();
            if let Some(ref trans) = *trans {
                let from_plane = self.scenes.get(&trans.from_scene)
                    .map(|s| s.render(area))
                    .unwrap_or_else(|| Plane::new(0, area.width, area.height));
                let to_plane = self.scenes.get(&trans.to_scene)
                    .map(|s| s.render(area))
                    .unwrap_or_else(|| Plane::new(0, area.width, area.height));
                return Self::blend_planes(from_plane, to_plane, trans.progress, trans.transition);
            }
        }

        if let Some(id) = self.stack.last() {
            if let Some(scene) = self.scenes.get(id) {
                return scene.render(area);
            }
        }
        Plane::new(0, area.width, area.height)
    }

    /// Advances any active transition by the given delta time in milliseconds.
    ///
    /// For apps with access to actual frame delta time, call this manually
    /// before `render()` for precise transition timing.
    pub fn tick_transition(&mut self, dt_ms: f32) -> bool {
        self.tick_transition_internal(dt_ms)
    }

    fn tick_transition_internal(&self, dt_ms: f32) -> bool {
        let mut trans_opt = self.transition.borrow_mut();
        if let Some(ref mut trans) = *trans_opt {
            trans.elapsed_ms += dt_ms;
            trans.progress = (trans.elapsed_ms / trans.duration_ms).min(1.0);

            if trans.progress >= 1.0 {
                *trans_opt = None;
                self.dirty.set(true);
                return false;
            }
            self.dirty.set(true);
            return true;
        }
        false
    }

    /// Returns true if a transition is currently active.
    pub fn is_transitioning(&self) -> bool {
        self.transition.borrow().is_some()
    }

    fn blend_planes(from: Plane, to: Plane, progress: f32, transition: SceneTransition) -> Plane {
        let width = from.width;
        let height = from.height;
        let mut result = Plane::new(0, width, height);

        // Safety: pre-fill result with cells from the "from" plane.
        // During transitions, narrow gaps can occur at the sliding edges
        // or when from/to have different dimensions, leaving cells at
        // Cell::default() (transparent + Color::Reset). Color::Reset
        // renders as the terminal's default background (typically white),
        // causing visible horizontal white lines across the screen.
        let copy_len = result.cells.len().min(from.cells.len());
        result.cells[..copy_len].copy_from_slice(&from.cells[..copy_len]);

        match transition {
            SceneTransition::Fade => {
                // Dithered crossfade: cells gradually switch from -> to
                let threshold = (progress * 10.0) as u8;
                for i in 0..from.cells.len().min(to.cells.len()) {
                    let use_to = (i % 10) < threshold as usize || progress >= 1.0;
                    result.cells[i] = if use_to { to.cells[i] } else { from.cells[i] };
                }
            }
            SceneTransition::SlideLeft => {
                let offset = (width as f32 * progress) as u16;
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let from_x = x.saturating_add(offset);
                        let to_x = x.saturating_sub(width - offset);
                        if from_x < width && from_x < from.width {
                            let from_idx = (y * from.width + from_x) as usize;
                            if from_idx < from.cells.len() {
                                result.cells[idx] = from.cells[from_idx];
                            }
                        }
                        if to_x < width && to_x < to.width {
                            let to_idx = (y * to.width + to_x) as usize;
                            if to_idx < to.cells.len() {
                                result.cells[idx] = to.cells[to_idx];
                            }
                        }
                    }
                }
            }
            SceneTransition::SlideRight => {
                let offset = (width as f32 * progress) as u16;
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let from_x = x.saturating_sub(offset);
                        let to_x = x.saturating_add(width - offset);
                        if from_x < width && from_x < from.width {
                            let from_idx = (y * from.width + from_x) as usize;
                            if from_idx < from.cells.len() {
                                result.cells[idx] = from.cells[from_idx];
                            }
                        }
                        if to_x < width && to_x < to.width {
                            let to_idx = (y * to.width + to_x) as usize;
                            if to_idx < to.cells.len() {
                                result.cells[idx] = to.cells[to_idx];
                            }
                        }
                    }
                }
            }
            SceneTransition::SlideUp => {
                let offset = (height as f32 * progress) as u16;
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let from_y = y.saturating_add(offset);
                        let to_y = y.saturating_sub(height - offset);
                        if from_y < height && from_y < from.height {
                            let from_idx = (from_y * from.width + x) as usize;
                            if from_idx < from.cells.len() {
                                result.cells[idx] = from.cells[from_idx];
                            }
                        }
                        if to_y < height && to_y < to.height {
                            let to_idx = (to_y * to.width + x) as usize;
                            if to_idx < to.cells.len() {
                                result.cells[idx] = to.cells[to_idx];
                            }
                        }
                    }
                }
            }
            SceneTransition::SlideDown => {
                let offset = (height as f32 * progress) as u16;
                for y in 0..height {
                    for x in 0..width {
                        let idx = (y * width + x) as usize;
                        let from_y = y.saturating_sub(offset);
                        let to_y = y.saturating_add(height - offset);
                        if from_y < height && from_y < from.height {
                            let from_idx = (from_y * from.width + x) as usize;
                            if from_idx < from.cells.len() {
                                result.cells[idx] = from.cells[from_idx];
                            }
                        }
                        if to_y < height && to_y < to.height {
                            let to_idx = (to_y * to.width + x) as usize;
                            if to_idx < to.cells.len() {
                                result.cells[idx] = to.cells[to_idx];
                            }
                        }
                    }
                }
            }
            _ => {
                // Instant switch — just show the target scene
                for i in 0..from.cells.len().min(to.cells.len()) {
                    result.cells[i] = to.cells[i];
                }
            }
        }
        result
    }

    // ── Input Delegation ────────────────────────────────────────────────────

    /// Delegates keyboard input to the current scene.
    pub fn handle_key(&mut self, key: KeyEvent) -> bool {
        if let Some(id) = self.stack.last().cloned() {
            if let Some(scene) = self.scenes.get_mut(&id) {
                return scene.handle_key(key);
            }
        }
        false
    }

    /// Delegates mouse input to the current scene.
    pub fn handle_mouse(&mut self, kind: MouseEventKind, col: u16, row: u16) -> bool {
        if let Some(id) = self.stack.last().cloned() {
            if let Some(scene) = self.scenes.get_mut(&id) {
                return scene.handle_mouse(kind, col, row);
            }
        }
        false
    }

    /// Propagates theme change to all registered scenes.
    pub fn on_theme_change(&mut self, theme: &Theme) {
        self.theme = Some(theme.clone());
        for scene in self.scenes.values_mut() {
            scene.on_theme_change(theme);
        }
        self.dirty.set(true);
    }

    /// Returns true if the current scene needs rendering.
    pub fn needs_render(&self) -> bool {
        if self.dirty.get() {
            return true;
        }
        if let Some(id) = self.stack.last() {
            if let Some(scene) = self.scenes.get(id) {
                return scene.needs_render();
            }
        }
        false
    }

    /// Marks the router as dirty.
    pub fn mark_dirty(&mut self) {
        self.dirty.set(true);
    }

    /// Clears the dirty flag.
    pub fn clear_dirty(&mut self) {
        self.dirty.set(false);
    }
}

impl Default for SceneRouter {
    fn default() -> Self {
        Self::new()
    }
}
