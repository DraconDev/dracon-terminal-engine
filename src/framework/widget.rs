//! Core widget trait for framework widgets.
//!
//! All framework widgets implement this trait to enable composition,
//! focus management, and event routing.

use crate::compositor::Plane;
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
}

/// Trait implemented by all framework widgets.
///
/// Provides a consistent interface for rendering, event handling,
/// and focus management across all widgets.
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
