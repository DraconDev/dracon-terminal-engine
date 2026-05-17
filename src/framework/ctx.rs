//! Application context for render and tick callbacks.
//!
//! [`Ctx`] provides access to the compositor, theme, animation manager, focus manager,
//! and dirty region tracker for use in render and tick callbacks.

use crate::compositor::{Compositor, Plane};
use crate::framework::command::{BoundCommand, CommandRunner};
use crate::framework::event_bus::EventBus;
use crate::framework::focus::FocusManager;
use crate::framework::animation::AnimationManager;
use crate::framework::dirty_regions::DirtyRegionTracker;
use crate::framework::scene_router::SceneRouter;
use crate::framework::theme::Theme;
use crate::framework::widget::WidgetId;
use crate::framework::widgets::split::SplitPane;
use crate::Terminal;
use ratatui::layout::Rect;
use std::cell::RefCell;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

/// Application context passed to every render and tick callback.
///
/// Provides access to the compositor, theme, animation manager, focus manager,
/// and dirty region tracker. Use it to add planes, manage focus, and mark
/// screen regions as dirty for the next render pass.
///
/// ## Example
///
/// ```ignore
/// app.run(|ctx| {
///     ctx.mark_dirty(0, 0, 80, 24); // mark entire screen dirty
///     let plane = my_widget.render(ctx.compositor().size().into());
///     ctx.add_plane(plane);
/// });
/// ```
pub struct Ctx<'a> {
    pub(crate) compositor: &'a mut Compositor,
    pub(crate) theme: &'a mut Theme,
    pub(crate) frame_count: u64,
    pub(crate) last_frame: &'a Instant,
    pub(crate) terminal: &'a mut Terminal<io::Stdout>,
    pub(crate) focus_manager: &'a mut FocusManager,
    pub(crate) animations: &'a mut AnimationManager,
    pub(crate) dirty_tracker: &'a mut DirtyRegionTracker,
    pub(crate) commands: &'a RefCell<Vec<BoundCommand>>,
    pub(crate) running: &'a AtomicBool,
    pub(crate) event_bus: &'a EventBus,
    pub(crate) scene_router: &'a mut SceneRouter,
}

impl<'a> Ctx<'a> {
    /// Adds a plane to the compositor.
    pub fn add_plane(&mut self, plane: Plane) {
        self.compositor.add_plane(plane);
    }

    /// Shows the terminal cursor (for text input widgets).
    pub fn show_cursor(&mut self) -> io::Result<()> {
        self.terminal.show_cursor()
    }

    /// Hides the terminal cursor (for non-text widgets during render).
    pub fn hide_cursor(&mut self) -> io::Result<()> {
        self.terminal.hide_cursor()
    }

    /// Sets the terminal cursor position.
    pub fn set_cursor(&mut self, col: u16, row: u16) -> io::Result<()> {
        self.terminal.set_cursor(col, row)
    }

    /// Temporarily restore terminal to normal mode for child processes.
    pub fn suspend_terminal(&mut self) -> io::Result<()> {
        self.terminal.suspend()
    }

    /// Re-enter raw mode + alternate screen after suspend_terminal().
    pub fn resume_terminal(&mut self) -> io::Result<()> {
        self.terminal.resume()?;
        // Force full redraw — terminal was externally cleared
        self.compositor.invalidate_last_frame();
        self.dirty_tracker.mark_all_dirty();
        Ok(())
    }

    /// Sets the focused widget by ID.
    pub fn set_focus(&mut self, id: WidgetId) {
        self.focus_manager.set_focus(id);
    }

    /// Returns the currently focused widget ID, if any.
    pub fn focused(&self) -> Option<WidgetId> {
        self.focus_manager.focused()
    }

    /// Returns the animation manager for managing toasts, progress bars, etc.
    pub fn animations(&self) -> &AnimationManager {
        self.animations
    }

    /// Returns the number of registered widgets.
    pub fn widget_count(&self) -> usize {
        self.compositor.widget_count()
    }

    /// Returns the number of planes in the compositor.
    pub fn plane_count(&self) -> usize {
        self.compositor.planes.len()
    }

    /// Returns the last frame duration in milliseconds.
    pub fn frame_time_ms(&self) -> f64 {
        self.compositor.last_frame_duration_ms()
    }

    /// Returns a mutable reference to the animation manager.
    pub fn animations_mut(&mut self) -> &mut AnimationManager {
        self.animations
    }

    /// Marks a screen region as dirty, so it will be re-rendered on the next frame.
    pub fn mark_dirty(&mut self, x: u16, y: u16, width: u16, height: u16) {
        self.dirty_tracker.mark_dirty(x, y, width, height);
    }

    /// Marks the entire screen as dirty, requiring a full refresh.
    pub fn mark_all_dirty(&mut self) {
        self.dirty_tracker.mark_all_dirty();
    }

    /// Returns true if a full screen refresh is needed.
    pub fn needs_full_refresh(&self) -> bool {
        self.dirty_tracker.needs_full_refresh()
    }

    /// Returns an immutable reference to the compositor.
    pub fn compositor(&self) -> &Compositor {
        self.compositor
    }

    /// Returns a mutable reference to the compositor.
    pub fn compositor_mut(&mut self) -> &mut Compositor {
        self.compositor
    }

    /// Returns a reference to the current theme.
    pub fn theme(&self) -> &Theme {
        self.theme
    }

    /// Sets the current theme. Use this in Pattern 2 apps (on_tick closures)
    /// to change the framework theme so all child widgets receive the new theme.
    /// The theme change is detected by `App::run()` after the tick callback
    /// and propagated to all widgets automatically.
    pub fn set_theme(&mut self, theme: Theme) {
        self.compositor.set_clear_color(theme.bg);
        *self.theme = theme;
    }

    /// Clears the entire terminal.
    pub fn clear(&mut self) {
        self.compositor.force_clear();
    }

    /// Returns the measured frames per second based on elapsed time and frame count.
    pub fn fps(&self) -> u64 {
        let frame_time = self.compositor.last_frame_duration_ms();
        if frame_time > 0.0 {
            (1000.0 / frame_time) as u64
        } else {
            0
        }
    }

    /// Splits the screen horizontally into two panes and passes them to the closure.
    pub fn split_h<F>(&mut self, f: F)
    where
        F: FnOnce(&mut SplitPane, &mut SplitPane),
    {
        let (w, h) = self.compositor.size();
        let split = SplitPane::new(crate::framework::widgets::split::Orientation::Horizontal)
            .ratio(0.5);
        let (r1, r2) = split.split(Rect::new(0, 0, w, h));
        let mut left = SplitPane::from_rect(r1);
        let mut right = SplitPane::from_rect(r2);
        f(&mut left, &mut right);
    }

    /// Splits the screen vertically into two panes and passes them to the closure.
    pub fn split_v<F>(&mut self, f: F)
    where
        F: FnOnce(&mut SplitPane, &mut SplitPane),
    {
        let (w, h) = self.compositor.size();
        let split = SplitPane::new(crate::framework::widgets::split::Orientation::Vertical)
            .ratio(0.5);
        let (r1, r2) = split.split(Rect::new(0, 0, w, h));
        let mut top = SplitPane::from_rect(r1);
        let mut bottom = SplitPane::from_rect(r2);
        f(&mut top, &mut bottom);
    }

    // ── Event Bus ───────────────────────────────────────────────────────────

    /// Publishes an event to the app's event bus.
    pub fn publish<E: std::any::Any + Clone>(&self, event: E) {
        self.event_bus.publish(event);
    }

    /// Subscribes to events of type `E` on the app's event bus.
    pub fn subscribe<E: std::any::Any + Clone, F>(&self, callback: F) -> crate::framework::event_bus::SubscriptionId
    where
        F: Fn(&E) + 'static,
    {
        self.event_bus.subscribe(callback)
    }

    /// Returns a reference to the event bus.
    pub fn event_bus(&self) -> &EventBus {
        self.event_bus
    }

    // ── Scene Router ────────────────────────────────────────────────────────

    /// Returns a mutable reference to the scene router.
    pub fn scene_router(&mut self) -> &mut SceneRouter {
        self.scene_router
    }

    /// Pushes a scene onto the navigation stack.
    pub fn push_scene(&mut self, id: &str) {
        self.scene_router.push(id);
    }

    /// Pops the current scene from the navigation stack.
    pub fn pop_scene(&mut self) -> bool {
        self.scene_router.pop()
    }

    /// Replaces the current scene.
    pub fn replace_scene(&mut self, id: &str) {
        self.scene_router.replace(id);
    }

    /// Navigates to a scene, clearing the stack.
    pub fn go_to_scene(&mut self, id: &str) {
        self.scene_router.go(id);
    }

    /// Lays out child rectangles using the given constraints within the screen area.
    ///
    /// This is a convenience method that uses the `Layout` engine to compute
    /// widget rectangles from constraint specifications (percentage, fixed, min, max, ratio).
    pub fn layout(&self, constraints: Vec<crate::framework::layout::Constraint>) -> Vec<Rect> {
        let (w, h) = self.compositor.size();
        let layout = crate::framework::layout::Layout::new(constraints);
        layout.layout(Rect::new(0, 0, w, h))
    }

    /// Runs a command synchronously and returns its output.
    ///
    /// Returns `(stdout, stderr, exit_code)`.
    pub fn run_command(&self, cmd: &str) -> (String, String, i32) {
        let runner = CommandRunner::new(cmd);
        runner.run_sync()
    }

    /// Returns all available commands registered with the app.
    pub fn available_commands(&self) -> Vec<BoundCommand> {
        self.commands.borrow().clone()
    }

    /// Stops the application event loop on the next iteration.
    pub fn stop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}