//! The terminal application framework.
//!
//! Provides [`App`] — a one-import entry point that owns the terminal,
//! compositor, input parsing, and event loop.
//!
//! # Example
//!
//! ```no_run
//! use dracon_terminal_engine::framework::prelude::*;
//!
//! App::new()
//!     .title("My App")
//!     .fps(30)
//!     .run(|ctx| {
//!         ctx.split_h(|left, right| {
//!             left.list(vec!["Item 1", "Item 2", "Item 3"], |item| {
//!                 println!("selected: {item}");
//!             });
//!             right.text("Hello, world!");
//!         });
//!     });
//! ```

pub mod app;
pub mod hitzone;
pub mod scroll;
pub mod theme;
pub mod widgets;

pub mod prelude {
    pub use crate::framework::{
        app::{App, Ctx},
        hitzone::{DragState, HitZone, HitZoneGroup},
        scroll::ScrollContainer,
        theme::Theme,
        widgets::*,
    };
    pub use crate::compositor::{Cell, Color, Compositor, Plane, Styles};
    pub use crate::input::event::{Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent};
    pub use crate::Terminal;
}