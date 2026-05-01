//! The terminal application framework.
//!
//! Provides `App` — a one-import entry point that owns the terminal,
//! compositor, input parsing, and event loop.
//!
//! # Example
//!
//! ```no_run
//! use dracon_terminal_engine::framework::prelude::*;
//! use dracon_terminal_engine::framework::widget::Widget;
//! use ratatui::layout::Rect;
//!
//! App::new().unwrap()
//!     .title("My App")
//!     .fps(30)
//!     .run(|ctx| {
//!         let (w, h) = ctx.compositor().size();
//!         let area = Rect::new(0, 0, w, h);
//!         let list = List::new(vec!["Item 1", "Item 2", "Item 3"]);
//!         ctx.add_plane(list.render(area));
//!     });
//! ```

pub mod animation;
pub mod app;
pub mod command;
pub mod dirty_regions;
pub mod dragdrop;
pub mod event_dispatcher;
pub mod focus;
pub mod hitzone;
pub mod layout;
pub mod scroll;
pub mod sixel;
/// The theme module.
pub mod theme;
pub mod widget;
pub mod widgets;

/// The prelude module.
pub mod prelude {
    pub use crate::compositor::{Cell, Color, Compositor, Plane, Styles};
    pub use crate::framework::{
        animation::{Animation, AnimationManager, Easing},
        app::{App, Ctx},
        command::{
            AppConfig, AreaConfig, BoundCommand, CommandRunner, LayoutConfig, LoggedLine,
            OutputParser, ParsedOutput, ParserConfig, WidgetConfig,
        },
        dirty_regions::{DirtyRegion, DirtyRegionTracker},
        dragdrop::{DragGhost, DragManager, DragPhase},
        focus::FocusManager,
        hitzone::{DragState, HitZone, HitZoneGroup, ScopedZone, ScopedZoneRegistry},
        layout::{Constraint, Layout},
        scroll::{ScrollContainer, ScrollState},
        theme::Theme,
        widgets::*,
    };
    pub use crate::input::event::{
        Event, KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent,
    };
    pub use crate::Terminal;
}
