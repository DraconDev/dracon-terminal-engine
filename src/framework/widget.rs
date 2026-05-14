//! Core widget trait for framework widgets.
//!
//! All framework widgets implement this trait to enable composition,
//! focus management, and event routing.

use serde_json::Value as JsonValue;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::compositor::Plane;
use crate::error::DraconError;
use crate::framework::command::{BoundCommand, ParsedOutput};
use crate::input::event::{KeyEvent, MouseEventKind};
use ratatui::layout::Rect;

/// Unique identifier for a widget (for event routing and state management).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct WidgetId(pub usize);

impl WidgetId {
    /// Creates a new `WidgetId` with the given numeric value.
    pub fn new(id: usize) -> Self {
        Self(id)
    }

    /// Creates a default `WidgetId` with value 0.
    pub fn default_id() -> Self {
        Self(0)
    }

    /// Generates a new unique `WidgetId` using an atomic counter.
    /// Use this in widget constructors for auto-assigned IDs.
    pub fn next() -> Self {
        static COUNTER: AtomicUsize = AtomicUsize::new(1);
        Self(COUNTER.fetch_add(1, Ordering::Relaxed))
    }
}

/// Trait implemented by all framework widgets.
///
/// Provides a consistent interface for rendering, event handling,
/// and focus management across all widgets.
///
/// ## Future decomposition
///
/// For a future 1.0 release, this trait may be decomposed into:
/// - `Renderable`: render, needs_render, mark_dirty, clear_dirty
/// - `Focusable`: focusable, on_focus, on_blur, cursor_position
/// - `Themable`: on_theme_change, current_theme
/// - `Commandable`: commands, apply_command_output
/// - `InputHandler`: handle_key, handle_mouse
///
/// `Widget` would remain as a convenience supertrait combining all of them.
pub trait Widget {
    /// Returns the unique identifier for this widget.
    fn id(&self) -> WidgetId;

    /// Returns the current area of this widget.
    fn area(&self) -> Rect;

    /// Sets the area of this widget.
    fn set_area(&mut self, area: Rect);

    /// Returns true if this widget can receive focus.
    fn focusable(&self) -> bool {
        true
    }

    /// Returns the z-index for layering (higher = on top).
    ///
    /// Recommended z-index ranges:
    ///
    /// | Range   | Layer                              |
    /// |---------|------------------------------------|
    /// | 0       | Background/base widgets            |
    /// | 5       | Content areas (panels, split panes) |
    /// | 10      | Interactive widgets (lists, forms, editors) |
    /// | 50      | Overlays (tooltips, dropdowns)      |
    /// | 100     | Modal dialogs                       |
    /// | 500     | Toasts/notifications                |
    /// | 9000    | Drag ghost (reserved)               |
    fn z_index(&self) -> u16 {
        0
    }

    /// Returns true if this widget needs to be rendered.
    /// When false, the widget is skipped during the render pass.
    fn needs_render(&self) -> bool {
        true
    }

    /// Marks the widget as dirty, so the next render pass will re-render it.
    /// Call this after state changes to ensure the widget re-renders.
    fn mark_dirty(&mut self) {}

    /// Clears the dirty flag after rendering.
    /// The render loop calls this automatically after a successful render.
    fn clear_dirty(&mut self) {}

    /// Returns the cursor position for text input widgets.
    /// Returns `None` if the widget does not show a cursor.
    fn cursor_position(&self) -> Option<(u16, u16)> {
        None
    }

    /// Renders the widget into a `Plane` at the given area.
    fn render(&self, area: Rect) -> Plane;

    /// Renders the widget directly into a target plane at the given offset.
    ///
    /// This is an optional optimization that allows widgets to render directly
    /// into a sub-region without allocating an intermediate plane. The default
    /// implementation calls `render()` then blits the result at the offset.
    ///
    /// Override this method to avoid the allocation when the widget can render
    /// directly into the target plane's coordinate system.
    fn draw_to(&mut self, target: &mut Plane, x: u16, y: u16) {
        let area = self.area();
        let plane = self.render(area);
        target.blit_from(&plane, x, y);
    }

    /// Called when the widget gains focus.
    fn on_focus(&mut self) {}

    /// Called when the widget loses focus.
    fn on_blur(&mut self) {}

    /// Called when the widget is added to the application.
    fn on_mount(&mut self) {}

    /// Called when the widget is removed from the application.
    fn on_unmount(&mut self) {}

    /// Sets the widget's ID.
    /// Called by `App::add_widget` to sync the App-assigned ID with the widget.
    fn set_id(&mut self, _id: WidgetId) {}

    /// Called when the theme is changed via `App::set_theme()`.
    /// Allows widgets to update their internal theme-dependent state.
    fn on_theme_change(&mut self, _theme: &crate::framework::theme::Theme) {}

    /// Returns the widget's current theme, if it manages its own theme state.
    ///
    /// Used by the framework to detect when a widget has changed its theme
    /// independently (e.g. Pattern-2 apps cycling themes via handle_key) and
    /// sync it back to the App framework. Returns `None` by default — widgets
    /// that manage their own theme should override this.
    fn current_theme(&self) -> Option<crate::framework::theme::Theme> {
        None
    }

    /// Handles a keyboard event.
    /// Returns `true` if the event was consumed, `false` if it should bubble.
    fn handle_key(&mut self, _key: KeyEvent) -> bool {
        false
    }

    /// Handles a mouse event within the widget's bounds.
    /// Returns `true` if the event was consumed.
    fn handle_mouse(&mut self, _kind: MouseEventKind, _col: u16, _row: u16) -> bool {
        false
    }

    /// Returns the list of commands this widget can execute.
    ///
    /// Each command binds to a CLI command and specifies how to parse its output.
    /// AI can enumerate these to know what actions are available.
    ///
    /// The default implementation returns an empty list.
    fn commands(&self) -> Vec<BoundCommand> {
        Vec::new()
    }

    /// Applies the parsed output of a bound command to this widget.
    ///
    /// Called automatically by the app tick loop when a widget's bound command
    /// is re-run after `refresh_seconds` has elapsed.
    ///
    /// The default implementation does nothing — widgets that bind commands
    /// override this to update their internal state from `ParsedOutput`.
    fn apply_command_output(&mut self, _output: &ParsedOutput) {}
}

/// Async lifecycle extension for widgets.
///
/// Separate from `Widget` because async methods are not dyn-compatible
/// (object-safe). Widgets that need async mount/unmount hooks should
/// implement both `Widget` and `AsyncWidget`.
#[cfg(feature = "async")]
pub trait AsyncWidget: Widget {
    /// Called when the widget is mounted (async variant).
    ///
    /// Use this for async initialization like loading resources or fetching data.
    fn on_mount_async(&mut self) -> impl std::future::Future<Output = ()> + Send + '_ {}

    /// Called when the widget is unmounted (async variant).
    ///
    /// Use this for async cleanup like saving state or closing connections.
    fn on_unmount_async(&mut self) -> impl std::future::Future<Output = ()> + Send + '_ {}
}

/// Trait for widgets that support state serialization to/from JSON.
///
/// Enables saving and restoring widget state for persistence, undo/redo,
/// or application state snapshots.
pub trait WidgetState {
    /// Returns a unique identifier for this widget's state, or `None` if
    /// this widget does not support serialization.
    ///
    /// The ID should be unique within the application context.
    fn state_id(&self) -> Option<&str>;

    /// Serializes the widget's current state to a JSON value.
    ///
    /// The returned value should be self-consistent and suitable for
    /// later restoration via `from_json()`.
    fn to_json(&self) -> JsonValue;

    /// Restores the widget's state from a JSON value.
    ///
    /// Returns `Ok(())` on success, or an error if the JSON is invalid
    /// or cannot be applied to this widget.
    fn apply_json(&mut self, json: &JsonValue) -> Result<(), DraconError>;
}
