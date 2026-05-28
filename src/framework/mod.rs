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
//! struct MyWidget { theme: Theme }
//!
//! impl Widget for MyWidget {
//!     fn id(&self) -> WidgetId { WidgetId::new(0) }
//!     fn area(&self) -> Rect { Rect::new(0, 0, 80, 24) }
//!     fn set_area(&mut self, _: Rect) {}
//!     fn needs_render(&self) -> bool { true }
//!     fn render(&self, area: Rect) -> Plane {
//!         let mut p = Plane::new(0, area.width, area.height);
//!         p.fill_bg(self.theme.bg);
//!         p
//!     }
//! }
//!
//! fn main() -> std::io::Result<()> {
//!     // App::new() can fail if the terminal cannot be initialized.
//!     let mut app = App::new()?;
//!     app.title("My App")
//!         .fps(30)
//!         .on_tick(|ctx, _tick| {
//!             let (w, h) = ctx.compositor().size();
//!             let area = Rect::new(0, 0, w, h);
//!             let list = List::new(vec!["Item 1", "Item 2", "Item 3"]);
//!             ctx.add_plane(list.render(area));
//!         })
//!         .run(|_| {});
//!     Ok(())
//! }
//! ```

pub mod animation;
pub mod app;
pub mod command;
pub mod ctx;
pub mod dirty_regions;
pub mod dragdrop;
pub mod event_bus;
pub mod event_dispatcher;
pub mod focus;
pub mod helpers;
pub mod hitzone;
pub mod i18n;
pub mod keybindings;
pub mod layout;
#[cfg(feature = "tracing")]
pub mod logging;
pub mod marquee;
pub mod plugin;
pub mod scroll;
#[cfg(feature = "sixel")]
pub mod sixel;
pub mod scene_router;
pub mod theme;
pub mod widget;
pub mod widget_container;
pub mod widgets;

/// The prelude module.
pub mod prelude {
    pub use crate::compositor::{Cell, CellPool, Color, Compositor, Plane, PoolConfig, Styles};
    pub use crate::error::DraconError;
    pub use crate::framework::widget::{Commandable, Focusable, InputHandler, Renderable, Themable, Widget, WidgetId, WidgetState};
    #[cfg(feature = "tracing")]
    pub use crate::frame_span;
    #[cfg(feature = "tracing")]
    pub use crate::frame_span_debug;
    pub use crate::framework::{
        animation::{Animation, AnimationManager, Easing},
        app::{App, Ctx, WidgetRef, WidgetRefMut},
        command::{
            AppConfig, AreaConfig, BoundCommand, CommandRunner, LayoutConfig, LoggedLine,
            OutputParser, ParsedOutput, ParserConfig, WidgetConfig,
        },
        dirty_regions::{DirtyRegion, DirtyRegionTracker},
        dragdrop::{DragGhost, DragManager, DragPhase},
        event_bus::{EventBus, Reactive, SubscriptionId},
        focus::FocusManager,
        helpers::{blit_to, draw_rounded_border, draw_text},
        hitzone::{DragState, HitZone, HitZoneGroup, ScopedZone, ScopedZoneRegistry},
        i18n::{tr, I18n, I18nError},
        keybindings::{KeybindingConfig, KeybindingSet, actions, resolve_keybindings},
        layout::{Constraint, Direction, Layout},
        marquee::{MarqueeRect, MarqueeState, render_marquee},
        plugin::{PluginRegistry, WidgetFactory},
        scroll::{ScrollContainer, ScrollState},
        scene_router::{NavigationEvent, Scene, SceneRouter},
        theme::Theme,
        widgets::*,
    };
    pub use crate::input::event::{
        Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseButton, MouseEvent,
        MouseEventKind,
    };
    #[cfg(feature = "tracing")]
    pub use tracing::instrument;
    pub use crate::Terminal;
    pub use ratatui::layout::Rect;
}
