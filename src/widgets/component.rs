//! Deprecated component bounds utilities.
//!
//! This module is deprecated. Use [`crate::framework::layout::Layout`] instead.

#![allow(deprecated)]

use crate::compositor::engine::Compositor;
use crate::input::event::Event;

/// Represents the rectangular bounds of a component on screen.
#[deprecated(since = "0.2.0", note = "Use framework::layout::Layout instead")]
#[derive(Debug, Clone, Copy, Default)]
pub struct Bounds {
    /// The x coordinate (column) of the top-left corner.
    pub x: u16,
    /// The y coordinate (row) of the top-left corner.
    pub y: u16,
    /// The width in columns.
    pub w: u16,
    /// The height in rows.
    pub h: u16,
}

#[allow(deprecated)]
impl Bounds {
    /// Creates a new Bounds with the given coordinates and dimensions.
    pub fn new(x: u16, y: u16, w: u16, h: u16) -> Self {
        Self { x, y, w, h }
    }

    /// Returns true if the given (col, row) is inside these bounds.
    pub fn contains(&self, col: u16, row: u16) -> bool {
        col >= self.x && col < self.x + self.w && row >= self.y && row < self.y + self.h
    }
}

/// The core trait for high-level semantic UI elements.
///
/// Components encapsulate rendering logic, allowing them to be composed
/// into complex layouts without manual cell manipulation.
///
/// **Deprecated:** Use the `Widget` trait from `framework::widget` instead.
/// This trait is not implemented by any framework widget and will be
/// removed in a future release.
#[deprecated(since = "0.2.0", note = "Use framework::widget::Widget instead")]
pub trait Component {
    /// Renders the component into the compositor at the specified bounds.
    ///
    /// # Arguments
    /// * `compositor` - The target rendering engine.
    /// * `bounds` - The rectangular area to render into.
    #[allow(deprecated)]
    fn render(&self, compositor: &mut Compositor, bounds: Bounds);

    /// Handles an input event.
    ///
    /// Returns `true` if the event was consumed (preventing propagation).
    #[allow(deprecated)]
    fn on_event(&mut self, _event: &Event, _bounds: Bounds) -> bool {
        false // Default: events pass through
    }

    /// Returns the preferred size of the component (width, height), if any.
    /// This is used for layout engines to determine optimal sizing.
    fn preferred_size(&self) -> (Option<u16>, Option<u16>) {
        (None, None)
    }
}
